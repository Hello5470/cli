mod list;
pub mod login;
mod logout;
mod switch;
pub mod types;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::state::State;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[clap(name = "ls", alias = "list")]
    List(list::Options),
    Login(login::Options),
    Logout(logout::Options),
    Switch(switch::Options),
}

#[derive(Debug, Parser)]
#[clap(name = "auth", about = "Authenticate with Hop")]
pub struct Options {
    #[clap(subcommand)]
    pub commands: Commands,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    match options.commands {
        Commands::Login(options) => login::handle(options, state).await,
        Commands::Logout(options) => logout::handle(options, state).await,
        Commands::Switch(options) => switch::handle(options, state).await,
        Commands::List(options) => {
            list::handle(&options, &state);
            Ok(())
        }
    }
}
