use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Insert { key: String, value: String },
    Get { key: String },
    Remove { key: String },
    Index { key: String },
}
