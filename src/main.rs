use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
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
            Commands::Init { template } => {
                // ファイルを作成する
                match File::create(template) {
                    Ok(_) => println!("File '{}' created successfully.", template),
                    Err(e) => eprintln!("Failed to create file '{}': {}", template, e),
                }
            }
            Commands::New { name } => {
                let config_dir = match get_config_dir() {
                    Some(path) => path,
                    None => {
                        eprintln!("Failed to find configuration directory.");
                        return;
                    }
                };

                // ディレクトリが存在しない場合は作成する
                std::fs::create_dir_all(&config_dir).unwrap();

                // 新しいテンプレートファイルを作成する
                let filename = config_dir.join(format!("{}.yaml", name));
                match File::create(&filename) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Err(e) = file.write_all(DEFAULT_YAML.as_bytes()) {
                            eprintln!("Failed to write to template file '{:?}': {}", filename, e);
                        } else {
                            println!("Template file '{:?}' created successfully.", filename);
                        }
                    }
                    Err(e) => eprintln!("Failed to create template file '{:?}': {}", filename, e),
                }
            }
            Commands::List => {
                // テンプレートの一覧を表示する
                println!("Available templates:");
                // TODO: ここでテンプレートの一覧を取得して表示する処理を実装する
            }
        }
    }

    if let Some(template_arg) = cli.template {
        let config_dir = match get_config_dir() {
            Some(path) => path,
            None => {
                eprintln!("Failed to find configuration directory.");
                return;
            }
        };

        let template_file_path = config_dir.join(format!("{}.yaml", template_arg));
        match std::fs::read_to_string(&template_file_path) {
            Ok(content) => {
                println!("{}", content);
                match serde_yaml::from_str::<Template>(&content) {
                    Ok(template) => {
                        println!("Template Name: {}", template.name);
                        println!("Description: {}", template.description);

                        let mut variable_map = HashMap::new();

                        for var in &cli.vars {
                            if let Some((key, value)) = var.split_once('=') {
                                variable_map
                                    .insert(key.trim().to_string(), value.trim().to_string());
                            }
                        }

                        if let Some(variables) = template.variables {
                            println!("Variables:");
                            for var in variables {
                                if variable_map.contains_key(&var) {
                                    println!("  {}: {}", var, variable_map[&var]);
                                    continue;
                                }
                                println!("  Enter value for {}: ", var);
                                io::stdout().flush().unwrap();

                                let mut input = String::new();
                                // ユーザーからの入力を受け取る
                                io::stdin().read_line(&mut input).unwrap();
                                let value = input.trim().to_string();

                                variable_map.insert(var.clone(), value);
                            }
                        }
                        println!("Items:");
                        generator::create_items(&template.items, Path::new("."), &variable_map);
                    }
                    Err(e) => eprintln!("Failed to parse template file '{:?}': {}", config_dir, e),
                }
            }
            Err(e) => eprintln!("Failed to read template file '{:?}': {}", config_dir, e),
        }
    }
}
