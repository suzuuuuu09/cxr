use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cx",
    version = "0.1.0",
    author = "suzuuuuu09",
    about = "A tool to generate a directory structure from a YAML template.",
    arg_required_else_help = true
)]
pub struct Cli {
    // サブコマンドを定義する
    #[command(subcommand)]
    pub command: Option<Commands>,

    // サブコマンドが定義されていなかったら第一引数をテンプレート名として扱う
    #[arg(help = "Name of the template to generate")]
    pub template: Option<String>,

    #[arg(short = 'v', long = "var", help = "Set a variable value (format: VAR=VALUE)", num_args = 1..)]
    pub vars: Vec<String>,

    #[arg(
        short = 'o',
        long = "output",
        help = "Output directory (default: current directory)"
    )]
    pub output: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new template file")]
    New { name: String },

    #[command(about = "Remove a template file", alias = "delete")]
    Remove { name: String },

    #[command(about = "List all available templates")]
    List,
}
