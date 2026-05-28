use clap::{Parser, Subcommand};
use std::fs::File;
use std::path::Path;

mod template;

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

fn create_items(items: &[template::TemplateItem], base_path: &Path) {
    for item in items {
        match item {
            template::TemplateItem::Directory {
                name,
                items: sub_items,
            } => {
                let dir_path = base_path.join(name);
                println!("Creating directory: {:?}", dir_path);
                match std::fs::create_dir_all(&dir_path) {
                    Ok(_) => println!("Directory '{:?}' created successfully.", dir_path),
                    Err(e) => eprintln!("Failed to create directory '{:?}': {}", dir_path, e),
                }
                if let Some(inner_items) = sub_items {
                    create_items(inner_items, &dir_path);
                }
            }
            template::TemplateItem::File { name, content } => {
                let file_path = base_path.join(name);
                println!("Creating file: {:?}", file_path);
                match File::create(&file_path) {
                    Ok(mut file) => {
                        use std::io::Write;
                        if let Some(content) = content {
                            if let Err(e) = file.write_all(content.as_bytes()) {
                                eprintln!("Failed to write to file '{:?}': {}", file_path, e);
                            } else {
                                println!("File '{:?}' created successfully.", file_path);
                            }
                        } else {
                            println!(
                                "File '{:?}' created successfully (empty content).",
                                file_path
                            );
                        }
                    }
                    Err(e) => eprintln!("Failed to create file '{:?}': {}", file_path, e),
                }
            }
        }
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

    if let Some(template_arg) = cli.template {
        let filename = format!("{}.yaml", template_arg);
        match std::fs::read_to_string(&filename) {
            Ok(content) => {
                println!("{}", content);
                match serde_yaml::from_str::<template::Template>(&content) {
                    Ok(template) => {
                        println!("Template Name: {}", template.name);
                        println!("Description: {}", template.description);
                        if let Some(variables) = template.variables {
                            println!("Variables: {:?}", variables);
                        }
                        println!("Items:");
                        create_items(&template.items, Path::new("."));
                    }
                    Err(e) => eprintln!("Failed to parse template file '{}': {}", filename, e),
                }
            }
            Err(e) => eprintln!("Failed to read template file '{}': {}", filename, e),
        }
    }
}
