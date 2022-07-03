use crate::state::State;
use crate::types::Base;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "hop project create", about = "🎇 Create a new project")]
pub struct CreateOptions {
    #[structopt(long = "name", help = "Name of the project")]
    name: Option<String>,
    #[structopt(long = "namespace", help = "Namespace of the project")]
    namespace: Option<String>,
    #[structopt(long = "default", help = "Set as default project")]
    default: bool,
}

#[derive(Debug, Serialize)]
struct CreateParams {
    icon: Option<String>,
    name: String,
    namespace: String,
}

// types for the API response
#[derive(Debug, Deserialize)]
struct ProjectRes {
    pub id: String,
}

#[derive(Debug, Deserialize)]
struct CreateResponse {
    pub project: ProjectRes,
}

#[derive(Debug, Deserialize)]
struct NamespaceRes {
    success: bool,
}

async fn create_project(params: CreateParams, state: State) -> Result<ProjectRes, std::io::Error> {
    let json = state
        .http
        .request::<Base<CreateResponse>>(
            "POST",
            "/projects",
            Some(serde_json::to_string(&params).unwrap()),
        )
        .await
        .expect("Error while creating project")
        .unwrap();

    Ok(json.data.project)
}

pub async fn handle_create(options: CreateOptions, mut state: State) -> Result<(), std::io::Error> {
    let namespace = match options.namespace {
        Some(namespace) => namespace,
        None => {
            // copy state to allow blocking move
            let blocking_state = state.clone();

            // this is large
            // it has to be in the blocking task for the validate and
            // reqwest blocking to work
            tokio::task::spawn_blocking(move || {
                // create the propmpt
                dialoguer::Input::<String>::new()
                    .with_prompt("Namespace of the project")
                    .validate_with({
                        let client = blocking_state.clone().http.sync_client();

                        move |input: &String| -> Result<(), String> {
                            let resp = client
                                .get(format!(
                                    "{}/projects/namespaces/{}",
                                    blocking_state.http.base_url, input
                                ))
                                .send()
                                .expect("Error while getting namespace info");

                            if resp.json::<NamespaceRes>().unwrap().success {
                                Ok(())
                            } else {
                                Err(format!("Namespace `{}` is already taken", input))
                            }
                        }
                    })
                    .interact_text()
                    .expect("Error while getting namespace")
            })
            .await
            .expect("Error while blocking on input")
        }
    };

    let name = match options.name {
        Some(name) => name,
        None => dialoguer::Input::<String>::new()
            .with_prompt("Name of the project")
            .interact_text()
            .unwrap(),
    };

    let params = CreateParams {
        name: name.clone(),
        namespace: namespace.clone(),
        icon: None,
    };

    let res = create_project(params, state.clone()).await?;

    if options.default {
        state.ctx.project = Some(res.id.clone());
        state.ctx.save().await?;
    }

    println!("Created project `{}` ({})", name, namespace);

    Ok(())
}
