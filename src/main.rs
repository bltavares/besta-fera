use std::{collections::HashMap, str::FromStr};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bollard::container;
use futures_util::TryStreamExt;
use poise::{
    serenity_prelude::{self as serenity, CreateEmbed},
    CreateReply,
};
use strum::VariantArray;

struct Data {
    docker: bollard::Docker,
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

trait MinecraftDocker {
    async fn minecraft_containers(&self) -> Result<HashMap<String, String>, Error>;
}

impl MinecraftDocker for bollard::Docker {
    async fn minecraft_containers(&self) -> Result<HashMap<String, String>, Error> {
        let containers = self
            .list_containers(Some(container::ListContainersOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?;

        let mut response = HashMap::with_capacity(containers.len());
        for container in containers {
            if let (Some(names), Some(status)) = (container.names, container.status) {
                if let Some(name) = names.first() {
                    let name = name.strip_prefix('/').unwrap_or(name);
                    if ValidContainers::from_str(name).is_ok() {
                        response.insert(name.to_string(), status.to_string());
                    }
                }
            }
        }

        Ok(response)
    }
}

#[derive(poise::ChoiceParameter, strum::VariantArray, strum::IntoStaticStr, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
enum ValidContainers {
    Velocity,
    Creative,
    Survival,
    OneBlock,
    SkyBlock,
}

/// Get the status of all minecraft servers
#[poise::command(slash_command)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    tracing::info!(operation = "status");
    let containers = ctx.data().docker.minecraft_containers().await?;
    let mut response = CreateEmbed::new().title("Minecraft server status");
    for name in ValidContainers::VARIANTS {
        let name: &str = name.into();
        if let Some(status) = containers.get(name) {
            response = response.field(name, status, false);
        } else {
            response = response.field(name, "Missing", false);
        }
    }
    ctx.send(CreateReply::default().embed(response)).await?;
    Ok(())
}

/// Start a minecraft server
#[poise::command(slash_command)]
async fn start(ctx: Context<'_>, server: ValidContainers) -> Result<(), Error> {
    let container: &str = server.into();
    tracing::info!(container, operation = "start");

    let result = ctx
        .data()
        .docker
        .start_container(container, None::<container::StartContainerOptions<String>>)
        .await;

    if result.is_ok() {
        ctx.say(format!("{container}: started")).await?;
    } else {
        ctx.say(format!("{container}: failed to start")).await?;
    }
    Ok(())
}

/// Stop a minecraft server
#[poise::command(slash_command)]
async fn stop(ctx: Context<'_>, server: ValidContainers) -> Result<(), Error> {
    let container: &str = server.into();
    tracing::info!(container, operation = "stop");

    let result = ctx.data().docker.stop_container(container, None).await;
    if result.is_ok() {
        ctx.say(format!("{container}: stopped")).await?;
    } else {
        ctx.say(format!("{container}: failed to stop")).await?;
    }
    Ok(())
}

/// Get the last 10 lines of logs of a minecraft server
#[poise::command(slash_command)]
async fn logs(ctx: Context<'_>, server: ValidContainers) -> Result<(), Error> {
    let container: &str = server.into();
    tracing::info!(container, operation = "logs");

    let logs = ctx
        .data()
        .docker
        .logs(
            container,
            Some(container::LogsOptions {
                stdout: true,
                stderr: true,
                timestamps: false,
                tail: "10",
                ..Default::default()
            }),
        )
        .map_ok(|output| output.to_string())
        .try_collect::<Vec<_>>()
        .await?;

    ctx.say(format!("```{}```", logs.join("\n"))).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let docker = bollard::Docker::connect_with_local_defaults().expect("Could not talk to docker");

    let intents = serenity::GatewayIntents::non_privileged();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![status(), start(), stop(), logs()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { docker })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
