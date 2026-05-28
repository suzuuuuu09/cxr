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
        long = "vars-file",
        help = "YAML or JSON file with variable mappings (non-interactive)"
    )]
    pub vars_file: Option<String>,

    #[arg(
        short = 'o',
        long = "output",
        help = "Output directory (default: current directory)"
    )]
    pub output: Option<String>,

    #[arg(
        short = 'n',
        long = "dry-run",
        help = "Preview generated items without writing files"
    )]
    pub dry_run: bool,

    #[arg(long = "force", conflicts_with_all = ["backup", "skip"], help = "Overwrite existing files without prompting")]
    pub force: bool,

    #[arg(long = "backup", conflicts_with_all = ["force", "skip"], help = "Backup existing files to .bak before writing")]
    pub backup: bool,

    #[arg(long = "skip", conflicts_with_all = ["force", "backup"], help = "Skip existing files without prompting")]
    pub skip: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Create a new template file")]
    New { name: String },

    #[command(about = "Remove a template file", alias = "delete")]
    Remove { name: String },

    #[command(about = "Select a template with fzf and generate")]
    Fzf,

    #[command(about = "List all available templates")]
    List,

    #[command(about = "Lint template files")]
    Lint {
        #[arg(value_name = "TEMPLATE")]
        name: Option<String>,

        #[arg(long = "all", help = "Lint all templates", conflicts_with = "name")]
        all: bool,
    },
}
