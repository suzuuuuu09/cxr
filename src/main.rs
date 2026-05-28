use clap::Parser;
use colored::*;
use inquire::Text;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod args;
mod generator;
mod template;

use args::{Cli, Commands};
use template::{Template, Variable};

const DEFAULT_YAML: &str = include_str!("./default.yaml");

struct TemplateInfo {
    stem: String,
    name: String,
    description: String,
}

fn main() {
    let cli = Cli::parse();

    // コマンドが指定されたときの処理
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::New { name } => handle_new_command(name),
            Commands::Remove { name } => handle_remove_command(name),
            Commands::Fzf => handle_fzf_command(&cli),
            Commands::List => handle_list_command(),
        }
    }

    // テンプレート名が指定されたときの生成処理
    if let Some(template_arg) = cli.template.as_deref() {
        handle_generate_command(template_arg, &cli);
    }
}

/// XDG_CONFIG_HOME環境変数を使用して設定ディレクトリを取得する関数
fn get_config_dir() -> Option<PathBuf> {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(val) => Some(Path::new(&val).join("cx")),
        Err(_) => dirs::home_dir().map(|path| path.join(".config").join("cx")),
    }
}

/// `cx new <name>` コマンドの処理
fn handle_new_command(name: &str) {
    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => {
            eprintln!(
                "{} Failed to find configuration directory.",
                "Error:".red().bold()
            );
            return;
        }
    };

    std::fs::create_dir_all(&config_dir).unwrap();
    let filename = config_dir.join(format!("{}.yaml", name));

    // すでに同名のテンプレートファイルが存在する場合は上書きの確認をする
    if filename.exists() {
        let prompt_msg = format!("Template '{}.yaml' already exists. Overwrite?", name);
        let ans = inquire::Confirm::new(&prompt_msg)
            .with_default(false)
            .prompt();

        match ans {
            Ok(true) => println!("{}", "Overwriting template...".yellow().dimmed()),
            _ => {
                println!(
                    "{}",
                    "Operation cancelled. Existing template was preserved.".yellow()
                );
                return;
            }
        }
    }

    // ファイルを作成してデフォルトのYAML内容を書き込む
    match File::create(&filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(DEFAULT_YAML.as_bytes()) {
                eprint_error(
                    &format!(
                        "Failed to write to template file '{}'",
                        display_name(&filename)
                    ),
                    &e.to_string(),
                );
            } else {
                println!(
                    "{} Template file '{}' created successfully.",
                    "Success:".green().bold(),
                    display_name(&filename)
                );
            }
        }
        Err(e) => eprint_error(
            &format!(
                "Failed to create template file '{}'",
                display_name(&filename)
            ),
            &e.to_string(),
        ),
    }
}

/// `cx remove <name>` コマンドの処理
fn handle_remove_command(name: &str) {
    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => {
            eprintln!(
                "{} Failed to find configuration directory.",
                "Error:".red().bold()
            );
            return;
        }
    };

    let filename = config_dir.join(format!("{}.yaml", name));

    if !filename.exists() {
        eprintln!(
            "{} Template file '{}' does not exist.",
            "Error:".red().bold(),
            display_name(&filename)
        );
        return;
    }

    let prompt_msg = format!(
        "Are you sure you want to delete template '{}'?",
        display_name(&filename)
    );
    let ans = inquire::Confirm::new(&prompt_msg)
        .with_default(false)
        .prompt();

    match ans {
        Ok(true) => {
            if let Err(e) = std::fs::remove_file(&filename) {
                eprint_error(
                    &format!(
                        "Failed to delete template file '{}'",
                        display_name(&filename)
                    ),
                    &e.to_string(),
                );
            } else {
                println!(
                    "{} Template file '{}' deleted successfully.",
                    "Success:".green().bold(),
                    display_name(&filename)
                );
            }
        }
        _ => {
            println!(
                "{}",
                "Operation cancelled. Template file was preserved.".yellow()
            );
        }
    }
}

/// `cx list` コマンドの処理
fn handle_list_command() {
    println!("{}", "Available templates:".bold().cyan());

    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => return,
    };

    for template in load_templates(&config_dir) {
        println!(
            "  {} {:<12} -> {} ({})",
            "-".cyan(),
            template.stem.green().bold(),
            template.name,
            template.description.dimmed()
        );
    }
}

/// `cx fzf` コマンドの処理
fn handle_fzf_command(cli: &Cli) {
    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => {
            eprintln!(
                "{} Failed to find configuration directory.",
                "Error:".red().bold()
            );
            return;
        }
    };

    let templates = load_templates(&config_dir);
    if templates.is_empty() {
        eprint_error(
            "No templates found.",
            "place a template under the config directory",
        );
        return;
    }

    let selected = match select_template_with_fzf(&templates) {
        Ok(name) => name,
        Err(err) => {
            eprint_error("Failed to select a template with fzf", &err);
            return;
        }
    };

    handle_generate_command(&selected, cli);
}

fn load_templates(config_dir: &Path) -> Vec<TemplateInfo> {
    let mut templates = Vec::new();

    let Ok(entries) = std::fs::read_dir(config_dir) else {
        return templates;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if Path::extension(&path).and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        let (name, desc) = std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_yaml::from_str::<serde_yaml::Value>(&content).ok())
            .map(|yaml| {
                let n = yaml.get("name").and_then(|v| v.as_str()).unwrap_or(stem);
                let d = yaml
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(No description provided)");
                (n.to_string(), d.to_string())
            })
            .unwrap_or_else(|| (stem.to_string(), "(Failed to read metadata)".to_string()));

        templates.push(TemplateInfo {
            stem: stem.to_string(),
            name,
            description: desc,
        });
    }

    templates
}

fn select_template_with_fzf(templates: &[TemplateInfo]) -> Result<String, String> {
    let mut child = Command::new("fzf")
        .arg("--prompt=Template> ")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "Failed to open fzf stdin".to_string())?;
        for template in templates {
            writeln!(
                stdin,
                "{}\t{}\t{}",
                template.stem, template.name, template.description
            )
            .map_err(|e| e.to_string())?;
        }
    }

    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err("fzf was cancelled or exited with an error".to_string());
    }

    let selection = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selection.is_empty() {
        return Err("No template was selected".to_string());
    }

    let stem = selection.split('\t').next().unwrap_or(&selection);
    Ok(stem.to_string())
}

/// テンプレート名が指定された場合の生成処理
fn handle_generate_command(template_arg: &str, cli: &Cli) {
    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => {
            eprintln!(
                "{} Failed to find configuration directory.",
                "Error:".red().bold()
            );
            return;
        }
    };

    let template_file_path = config_dir.join(format!("{}.yaml", template_arg));
    let content = match std::fs::read_to_string(&template_file_path) {
        Ok(c) => c,
        Err(e) => {
            eprint_error(
                &format!("Failed to read template file '{:?}'", template_file_path),
                &e.to_string(),
            );
            return;
        }
    };

    let template = match serde_yaml::from_str::<Template>(&content) {
        Ok(t) => t,
        Err(e) => {
            eprint_error(
                &format!("Failed to parse template file '{:?}'", template_file_path),
                &e.to_string(),
            );
            return;
        }
    };

    println!("{} {}", "Template:".cyan().bold(), template.name.bold());
    println!("{} {}\n", "Description:".cyan(), template.description);

    // 変数マップの作成
    let mut variable_map = HashMap::new();
    for var in &cli.vars {
        if let Some((key, value)) = var.split_once('=') {
            variable_map.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    // 対話プロンプトで変数を取得
    if let Some(variables) = template.variables {
        if !collect_variables(variables, &mut variable_map) {
            return; // ユーザーが中断した場合は終了
        }
    }

    if cli.dry_run {
        println!("{}", "Dry run: no files will be written.".yellow().dimmed());
    }
    println!("\n{}", "Generating items...".bold().dimmed());
    let target_dir = cli.output.as_deref().unwrap_or(".");
    let target_path = Path::new(target_dir);

    let overwrite = overwrite_strategy(cli);
    std::fs::create_dir_all(target_path).unwrap();
    generator::create_items(
        &template.items,
        target_path,
        &variable_map,
        cli.dry_run,
        overwrite,
    );
    println!("{}", "\nDone!".green().bold());
}

/// ユーザーから変数の入力を集めるヘルパー関数（中断されたら false を返す）
fn collect_variables(variables: Vec<Variable>, variable_map: &mut HashMap<String, String>) -> bool {
    for var in variables {
        let var_name = var.name();

        if variable_map.contains_key(&var_name) {
            continue;
        }

        let prompt_msg = format!("Enter value for {}:", var_name);
        let mut text_prompt = Text::new(&prompt_msg);

        let default_val = var.default_value();
        if let Some(ref val) = default_val {
            text_prompt = text_prompt.with_default(val);
        }

        match text_prompt.prompt() {
            Ok(input) => match resolve_variable_input(&input, default_val) {
                Ok(value) => {
                    variable_map.insert(var_name, value);
                }
                Err(err) => {
                    eprint_error(&format!("Variable '{}' cannot be empty", var_name), err);
                    return false;
                }
            },
            Err(inquire::InquireError::OperationInterrupted)
            | Err(inquire::InquireError::OperationCanceled) => {
                println!("\n{}", "Operation cancelled.".red());
                return false;
            }
            Err(e) => {
                eprint_error(
                    &format!("Failed to get input for variable '{}'", var_name),
                    &e.to_string(),
                );
                return false;
            }
        }
    }

    println!("\n{}", "Variables:".bold().cyan());
    for (key, val) in variable_map.iter() {
        println!("  {} {}: {}", "->".cyan(), key, val.green());
    }
    true
}

fn eprint_error(context: &str, err: &str) {
    eprintln!("{} {}: {}", "Error:".red().bold(), context, err);
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| path.display().to_string())
}

fn resolve_variable_input(
    input: &str,
    default_value: Option<String>,
) -> Result<String, &'static str> {
    let input = input.trim();

    if input.is_empty() {
        default_value.ok_or("a value is required")
    } else {
        Ok(input.to_string())
    }
}

fn overwrite_strategy(cli: &Cli) -> generator::OverwriteStrategy {
    if cli.force {
        generator::OverwriteStrategy::Force
    } else if cli.backup {
        generator::OverwriteStrategy::Backup
    } else if cli.skip {
        generator::OverwriteStrategy::Skip
    } else {
        generator::OverwriteStrategy::Prompt
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_variable_input;

    #[test]
    fn resolve_variable_input_rejects_blank_without_default() {
        assert!(resolve_variable_input("   ", None).is_err());
    }

    #[test]
    fn resolve_variable_input_uses_default_for_blank_input() {
        assert_eq!(
            resolve_variable_input("   ", Some("fallback".to_string())).unwrap(),
            "fallback"
        );
    }

    #[test]
    fn resolve_variable_input_trims_non_blank_input() {
        assert_eq!(resolve_variable_input("  value  ", None).unwrap(), "value");
    }
}
