use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "deeb-server")]
#[command(about = "A lightweight realtime server with built in auth and access control.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create a default rules.rhai file
    InitRules {
        /// Path to create the rules file
        #[arg(default_value = "rules.rhai")]
        path: String,
    },

    /// Start the server
    Serve {
        #[arg(long, short = 'H', default_value = "127.0.0.1")]
        host: String,

        #[arg(long, short, default_value = "8080")]
        port: u16,

        /// Path to the rules file
        #[arg(long, default_value = "rules.rhai")]
        rules: Option<String>,

        /// The name of the instance/json file to save data to.
        #[arg(long, default_value = "rules.rhai")]
        instance_name: Option<String>,
    },
}
