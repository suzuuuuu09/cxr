use clap::Parser;
use colored::*;
use inquire::Text;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

mod args;
mod generator;
mod template;

use args::{Cli, Commands};
use template::Template;

const DEFAULT_YAML: &str = include_str!("./default.yaml");

// XDG_CONFIG_HOME環境変数を使用して設定ディレクトリを取得する関数
fn get_config_dir() -> Option<PathBuf> {
    match std::env::var("XDG_CONFIG_HOME") {
        Ok(val) => Some(Path::new(&val).join("cx")),
        Err(_) => dirs::home_dir().map(|path| path.join(".config").join("cx")),
    }
}

fn main() {
    // コマンドライン引数を解析する
    let cli = Cli::parse();

    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::New { name } => {
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

                // ディレクトリが存在しない場合は作成する
                std::fs::create_dir_all(&config_dir).unwrap();

                // 新しいテンプレートファイルを作成する
                let filename = config_dir.join(format!("{}.yaml", name));
                match File::create(&filename) {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(DEFAULT_YAML.as_bytes()) {
                            eprint_error(
                                &format!("Failed to write to template file '{:?}'", filename),
                                &e.to_string(),
                            );
                        } else {
                            println!(
                                "{} Template file '{:?}' created successfully.",
                                "Success:".green().bold(),
                                filename
                            );
                        }
                    }
                    Err(e) => eprint_error(
                        &format!("Failed to create template file '{:?}'", filename),
                        &e.to_string(),
                    ),
                }
            }
            Commands::List => {
                // 設定ディレクトリを取得
                println!("{}", "Available templates:".bold().cyan());

                if let Some(config_dir) = get_config_dir() {
                    // ディレクトリ内のファイルを読み込む
                    if let Ok(entries) = std::fs::read_dir(config_dir) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            // 拡張子が "yaml" のファイルだけを対象にする
                            if path.extension().is_some_and(|ext| ext == "yaml") {
                                // ファイル名（拡張子なし）を綺麗に表示
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    let (name, desc) = std::fs::read_to_string(&path)
                                        .ok()
                                        .and_then(|content| {
                                            // 辞書型（Value）として大雑把に読み込む
                                            serde_yaml::from_str::<serde_yaml::Value>(&content).ok()
                                        })
                                        .map(|yaml| {
                                            let n = yaml
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or(stem); // nameが無ければファイル名を使う
                                            let d = yaml
                                                .get("description")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("(No description provided)");
                                            (n.to_string(), d.to_string())
                                        })
                                        // 万が一ファイルが壊れていたらデフォルト値を返す
                                        .unwrap_or_else(|| {
                                            (
                                                stem.to_string(),
                                                "(Failed to read metadata)".dimmed().to_string(),
                                            )
                                        });

                                    println!(
                                        "  {} {:<12} -> {} ({})",
                                        "-".cyan(),
                                        stem.green().bold(),
                                        name,
                                        desc.dimmed()
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(template_arg) = cli.template {
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
        match std::fs::read_to_string(&template_file_path) {
            Ok(content) => match serde_yaml::from_str::<Template>(&content) {
                Ok(template) => {
                    println!("{} {}", "Template:".cyan().bold(), template.name.bold());
                    println!("{} {}\n", "Description:".cyan(), template.description);

                    let mut variable_map = HashMap::new();

                    for var in &cli.vars {
                        if let Some((key, value)) = var.split_once('=') {
                            variable_map.insert(key.trim().to_string(), value.trim().to_string());
                        }
                    }

                    if let Some(variables) = template.variables {
                        for var in variables {
                            if variable_map.contains_key(&var) {
                                continue;
                            }
                            let prompt_msg = format!("Enter value for {}:", var);
                            match Text::new(&prompt_msg).prompt() {
                                Ok(value) => {
                                    variable_map.insert(var.clone(), value.trim().to_string());
                                }
                                Err(_) => {
                                    eprintln!("{}", "Operation cancelled.".red());
                                    return;
                                }
                            }
                        }

                        println!("\n{}", "Variables:".bold().cyan());
                        for (key, val) in &variable_map {
                            println!("  {} {}: {}", "->".cyan(), key, val.green());
                        }
                    }

                    println!("\n{}", "Generating items...".bold().dimmed());
                    generator::create_items(&template.items, Path::new("."), &variable_map);
                    println!("{}", "\nDone!".green().bold());
                }
                Err(e) => eprint_error(
                    &format!("Failed to parse template file '{:?}'", template_file_path),
                    &e.to_string(),
                ),
            },
            Err(e) => eprint_error(
                &format!("Failed to read template file '{:?}'", template_file_path),
                &e.to_string(),
            ),
        }
    }
}

fn eprint_error(context: &str, err: &str) {
    eprintln!("{} {}: {}", "Error:".red().bold(), context, err);
}
