use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "ovpn")]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Config {
        #[command(subcommand)]
        commands: ConfigCommand
    },

    Session {
        #[command(subcommand)]
        commands: SessionCommand
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    List,
    Import {
        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        path: String,
    },
    Export {
        #[arg(short, long)]
        name: String,
    },
    Delete {
        #[arg(short, long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SessionCommand {
    Start {
        #[arg(short, long)]
        name: String,
    },
    Stop,
    Status,
}