use clap::Parser;

#[derive(Parser, Debug)]
pub struct Cli {
    #[arg(long, short = 'H', default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, short, default_value = "8080")]
    pub port: u16,
}
