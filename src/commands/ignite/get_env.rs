use std::io::Write;

use anyhow::{ensure, Result};
use clap::Parser;

use crate::commands::ignite::utils::{format_deployments, get_all_deployments, get_deployment};
use crate::commands::secrets::utils::get_secret_name;
use crate::state::State;

#[derive(Debug, Parser)]
#[clap(about = "Get current deployments env values")]
#[group(skip)]
pub struct Options {
    #[clap(help = "ID of the deployment to get env values")]
    pub deployment: Option<String>,
}

pub async fn handle(options: Options, state: State) -> Result<()> {
    let deployment = match options.deployment {
        Some(id) => get_deployment(&state.http, &id).await?,

        None => {
            let project_id = state.ctx.current_project_error()?.id;

            let deployments = get_all_deployments(&state.http, &project_id).await?;
            ensure!(!deployments.is_empty(), "No deployments found");
            let deployments_fmt = format_deployments(&deployments, false);

            let idx = dialoguer::Select::new()
                .with_prompt("Select a deployment")
                .items(&deployments_fmt)
                .default(0)
                .interact()?;

            deployments[idx].clone()
        }
    };

    let mut buff = vec![];

    for (key, value) in deployment.config.env {
        let value = if let Some(secret_name) = get_secret_name(&value) {
            format!("{{{secret_name}}}")
        } else {
            value
        };

        writeln!(buff, "{key}={value}")?;
    }

    print!("{}", String::from_utf8(buff)?);

    Ok(())
}
