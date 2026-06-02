use clap::Parser;
use colored::*;

mod vars;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

mod args;
mod generator;
mod hooks;
mod lint;
mod overwrite;
mod template;

use args::{Cli, Commands};
use template::{Template, load_template, template_name_matches};
mod vars_file;

const DEFAULT_YAML: &str = include_str!("./default.yaml");

struct TemplateInfo {
    stem: String,
    name: String,
    description: String,
    tags: Vec<String>,
    raw: Template,
}

fn main() {
    let cli = Cli::parse();

    // コマンドが指定されたときの処理
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::New { name } => handle_new_command(name),
            Commands::Remove { name } => handle_remove_command(name),
            Commands::Fzf => handle_fzf_command(&cli),
            Commands::List { tag, search } => {
                handle_list_command(tag.as_deref(), search.as_deref())
            }
            Commands::Lint { name, all } => handle_lint_command(name.as_deref(), *all),
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

    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        eprint_error(
            &format!(
                "Failed to prepare configuration directory '{}'",
                display_name(&config_dir)
            ),
            &e.to_string(),
        );
        return;
    }
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
fn handle_list_command(tag: Option<&str>, search: Option<&str>) {
    println!("{}", "Available templates:".bold().cyan());

    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => return,
    };

    for template in load_templates(&config_dir)
        .into_iter()
        .filter(|template| {
            tag.map(|needle| {
                template
                    .tags
                    .iter()
                    .any(|value| value.eq_ignore_ascii_case(needle))
            })
            .unwrap_or(true)
        })
        .filter(|template| {
            search
                .map(|query| template_name_matches(&template.raw, query))
                .unwrap_or(true)
        })
    {
        let tag_display = if template.tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", template.tags.join(", "))
        };
        println!(
            "  {} {:<12} -> {} ({}){}",
            "-".cyan(),
            template.stem.green().bold(),
            template.name,
            template.description.dimmed(),
            tag_display
        );
    }
}

/// `cx lint <template>` / `cx lint --all` の処理
fn handle_lint_command(name: Option<&str>, all: bool) {
    let config_dir = match get_config_dir() {
        Some(path) => path,
        None => {
            eprintln!(
                "{} Failed to find configuration directory.",
                "Error:".red().bold()
            );
            std::process::exit(1);
        }
    };

    let template_names = if all {
        let paths = list_template_files(&config_dir);
        if paths.is_empty() {
            eprint_error(
                "No templates found.",
                "place a template under the config directory",
            );
            std::process::exit(1);
        }
        paths
    } else if let Some(template_name) = name {
        let path = config_dir.join(format!("{}.yaml", template_name));
        if !path.exists() {
            eprint_error(
                &format!("Template file '{}' does not exist.", display_name(&path)),
                "use `cx list` to see available templates",
            );
            std::process::exit(1);
        }
        vec![path]
    } else {
        eprint_error(
            "Missing template name.",
            "use `cx lint <template>` or `cx lint --all`",
        );
        std::process::exit(2);
    };

    let mut total_errors = 0;
    for path in template_names {
        let Some(template_name) = path.file_stem().and_then(|name| name.to_str()) else {
            continue;
        };
        match load_template(&config_dir, template_name) {
            Ok(template) => {
                let report = lint::lint_template(&template);
                if report.is_empty() {
                    println!("  {} {}", "OK:".green().bold(), display_name(&path));
                    continue;
                }

                total_errors += report.len();
                eprintln!("{} {}", "Lint errors in".red().bold(), display_name(&path));
                for err in report {
                    eprintln!("  - {}", err);
                }
            }
            Err(err) => {
                total_errors += 1;
                eprintln!("{} {}", "Lint errors in".red().bold(), display_name(&path));
                eprintln!("  - {}", err);
            }
        }
    }

    if total_errors > 0 {
        eprintln!(
            "{} lint failed ({} issues).",
            "Error:".red().bold(),
            total_errors
        );
        std::process::exit(1);
    }

    println!("{} lint passed.", "Success:".green().bold());
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

        let raw = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_yaml::from_str::<Template>(&content).ok())
        {
            Some(template) => template,
            None => continue,
        };

        templates.push(TemplateInfo {
            stem: stem.to_string(),
            name: raw.name.clone(),
            description: raw.description.clone(),
            tags: raw.tags.clone().unwrap_or_default(),
            raw,
        });
    }

    templates
}

fn list_template_files(config_dir: &Path) -> Vec<PathBuf> {
    let mut templates = Vec::new();

    let Ok(entries) = std::fs::read_dir(config_dir) else {
        return templates;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if Path::extension(&path).and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        templates.push(path);
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
                "{}\t{}\t{}\t{}",
                template.stem,
                template.name,
                template.description,
                template.tags.join(", ")
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
    let template = match load_template(&config_dir, template_arg) {
        Ok(template) => template,
        Err(e) => {
            eprint_error(
                &format!("Failed to load template file '{:?}'", template_file_path),
                &e,
            );
            return;
        }
    };

    println!("{} {}", "Template:".cyan().bold(), template.name.bold());
    println!("{} {}\n", "Description:".cyan(), template.description);

    // 変数マップの作成
    let mut variable_map = HashMap::new();

    // CLI vars (highest precedence)
    match vars::parse_cli_vars(&cli.vars) {
        Ok(map) => {
            variable_map.extend(map);
        }
        Err(e) => {
            eprint_error("Failed to parse -v values", &e);
            return;
        }
    }

    // vars-file (lower precedence than CLI but higher than interactive)
    if let Some(vf) = &cli.vars_file {
        match vars_file::load_vars_from_file(vf) {
            Ok(map) => {
                for (k, v) in map {
                    variable_map.entry(k).or_insert(v);
                }
            }
            Err(e) => {
                eprint_error("Failed to load vars-file", &e);
                return;
            }
        }
    }

    // 対話プロンプトで変数を取得
    if let Some(variables) = template.variables.clone() {
        // If vars-file provided, skip interactive prompts for variables already present
        if !vars::collect_variables(variables, &mut variable_map) {
            return; // ユーザーが中断した場合は終了
        }
    }

    let target_dir = cli.output.as_deref().unwrap_or(".");
    let target_path = Path::new(target_dir);

    if let Err(e) = prepare_output_dir(target_path, cli.dry_run) {
        eprint_error(
            &format!(
                "Failed to prepare output directory '{}'",
                target_path.display()
            ),
            &e,
        );
        return;
    }

    if cli.dry_run {
        println!("{}", "Dry run: no files will be written.".yellow().dimmed());
    }

    if cli.dry_run {
        println!(
            "{}",
            "Dry run: hooks will not be executed.".yellow().dimmed()
        );
    } else if let Some(hook) = template.pre_hook.as_deref() {
        if let Err(err) = hooks::run_hook(
            "pre_hook",
            hook,
            target_path,
            &variable_map,
            &template.name,
            &template.description,
        ) {
            eprint_error("Failed to run pre_hook", &err);
            std::process::exit(1);
        }
    }
    println!("\n{}", "Generating items...".bold().dimmed());

    let overwrite = overwrite_strategy(cli);
    generator::create_items(
        &template.items,
        target_path,
        &variable_map,
        cli.dry_run,
        overwrite,
    );

    if !cli.dry_run {
        if let Some(hook) = template.post_hook.as_deref() {
            if let Err(err) = hooks::run_hook(
                "post_hook",
                hook,
                target_path,
                &variable_map,
                &template.name,
                &template.description,
            ) {
                eprint_error("Failed to run post_hook", &err);
                std::process::exit(1);
            }
        }
    }
    println!("{}", "\nDone!".green().bold());
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

fn overwrite_strategy(cli: &Cli) -> overwrite::OverwriteStrategy {
    if cli.force {
        overwrite::OverwriteStrategy::Force
    } else if cli.backup {
        overwrite::OverwriteStrategy::Backup
    } else if cli.skip {
        overwrite::OverwriteStrategy::Skip
    } else {
        overwrite::OverwriteStrategy::Prompt
    }
}

fn prepare_output_dir(target_path: &Path, dry_run: bool) -> Result<(), String> {
    if dry_run {
        return Ok(());
    }

    std::fs::create_dir_all(target_path).map_err(|e| e.to_string())
}

// hooks moved to src/hooks.rs

#[cfg(test)]
mod tests {
    use super::prepare_output_dir;
    use crate::vars::resolve_variable_input;
    use std::env::temp_dir;
    use std::fs;

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

    #[test]
    fn prepare_output_dir_skips_creation_on_dry_run() {
        let mut path = temp_dir();
        path.push("cx_test_dry_run_output");
        fs::remove_dir_all(&path).ok();

        prepare_output_dir(&path, true).unwrap();

        assert!(!path.exists());
    }
}
