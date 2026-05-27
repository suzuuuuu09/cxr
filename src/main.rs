use clap::{Parser, Subcommand};
use std::fs::File;

#[derive(Parser)]
#[command(
    name = "cx",
    version = "0.1.0",
    author = "suzuuuuu09",
    about = "A tool to generate a directory structure from a TOML template."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long, default_value = "cx.toml")]
        template: String,
    },
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
    }
}
