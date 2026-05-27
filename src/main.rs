use clap::{Parser, Subcommand};
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
    command: Commands,

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

fn main() {
    // コマンドライン引数を解析する
    let cli = Cli::parse();

    match &cli.command {
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
