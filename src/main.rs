use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs::File;

const DEFAULT_YAML: &str = include_str!("./default.yaml");

#[derive(Parser)]
#[command(
    name = "cx",
    version = "0.1.0",
    author = "suzuuuuu09",
    about = "A tool to generate a directory structure from a TOML template."
)]
struct Cli {
    // サブコマンドを定義する
    #[command(subcommand)]
    command: Option<Commands>,

    // サブコマンドが定義されていなかったら第一引数をテンプレート名として扱う
    #[arg(help = "Name of the template to generate")]
    template: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long, default_value = "cx.toml")]
        template: String,
    },

    #[command(about = "Create a new template file")]
    New { name: String },

    #[command(about = "List all available templates")]
    List,
}

#[derive(Debug, Serialize, Deserialize)]
struct Template {
    name: String,
    description: String,
    variables: Option<Vec<String>>,
    items: Vec<TemplateItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum TemplateItem {
    Directory {
        name: String,
    },
    File {
        name: String,
        content: Option<String>,
    },
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
                // 新しいテンプレートファイルを作成する
                let filename = format!("{}.yaml", name);
                match File::create(&filename) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Err(e) = file.write_all(DEFAULT_YAML.as_bytes()) {
                            eprintln!("Failed to write to template file '{}': {}", filename, e);
                        } else {
                            println!("Template file '{}' created successfully.", filename);
                        }
                    }
                    Err(e) => eprintln!("Failed to create template file '{}': {}", filename, e),
                }
            }
            Commands::List => {
                // テンプレートの一覧を表示する
                println!("Available templates:");
                // TODO: ここでテンプレートの一覧を取得して表示する処理を実装する
            }
        }
    }

    if let Some(template) = cli.template {
        let filename = format!("{}.yaml", template);
        match std::fs::read_to_string(&filename) {
            Ok(content) => {
                println!("{}", content);
                match serde_yaml::from_str::<Template>(&content) {
                    Ok(template) => {
                        println!("Template Name: {}", template.name);
                        println!("Description: {}", template.description);
                        if let Some(variables) = template.variables {
                            println!("Variables: {:?}", variables);
                        }
                        println!("Items:");
                        for item in template.items {
                            match item {
                                TemplateItem::Directory { name } => {
                                    println!("  - Directory: {}", name);
                                }
                                TemplateItem::File { name, content } => {
                                    println!("  - File: {}", name);
                                    if let Some(content) = content {
                                        println!("    Content: {}", content);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Failed to parse template file '{}': {}", filename, e),
                }
            }
            Err(e) => eprintln!("Failed to read template file '{}': {}", filename, e),
        }
    }
}
