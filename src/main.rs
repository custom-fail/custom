use std::sync::Arc;
use crate::context::Context;
use dotenv::dotenv;
use tokio::task::JoinHandle;
use tracing::info;
use twilight_http::Client;
use crate::gateway::clients::{DiscordClients, LoadDiscordClients};

all_macro!(
    cfg(feature = "gateway");
    mod events;
    mod links;
    mod bucket;
);

mod context;

#[cfg(any(feature = "gateway", feature = "custom-clients", feature = "tasks"))]
mod gateway;

#[cfg(feature = "tasks")]
mod tasks;
mod application;
mod commands;
mod database;
mod models;
pub mod utils;
mod server;
mod tracing_init;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_init::init();
    info!("starting app");

    let discord_token = env_unwrap!("DISCORD_TOKEN");
    let main_http = Arc::new(Client::new(discord_token.to_owned()));
    let current_user = main_http
        .current_user()
        .await
        .unwrap()
        .model()
        .await
        .unwrap();
    info!("current user fetched: {}#{}", current_user.name, current_user.discriminator);
    let application_id = current_user.id.cast();

    let context = Arc::new(Context::new(application_id).await);
    let cloned_context = context.clone();
    tokio::spawn(async move {
        let context = cloned_context;
        context.redis.watch_config_updates(&context.mongodb).await.unwrap();
    });

    let mut threads: Vec<JoinHandle<()>> = vec![];

    let discord_clients = DiscordClients::load(&context.mongodb).await.unwrap();

    #[cfg(feature = "tasks")]
    threads.push(tasks::run(
        context.mongodb.to_owned(),
        discord_clients.to_owned(),
        main_http.to_owned()
    ));

    #[cfg(any(feature = "custom-clients"))]
    threads.append(&mut discord_clients.start(context.to_owned()));

    #[cfg(feature = "gateway")]
    {
        use crate::gateway::shard::connect_shards;

        let run = tokio::spawn(connect_shards(
            ("main".to_string(), Arc::new(
                Client::new(discord_token.to_owned())
            )),
            context.to_owned()
        ));

        threads.push(run);
    }


    #[cfg(any(feature = "api", feature = "http-interactions"))]
    {
        const INVALID_PUBLIC_KEY: &str = "PUBLIC_KEY provided in .env is invalid";

        #[cfg(feature = "http-interactions")]
        let public_key = {
            let public_key = env_unwrap!("PUBLIC_KEY");
            let mut bytes_slice = [0; 32];
            let pbk_bytes = hex::decode(public_key.as_str())
                .expect(INVALID_PUBLIC_KEY);
            bytes_slice.copy_from_slice(&pbk_bytes);
            ed25519_dalek::VerifyingKey::from_bytes(&bytes_slice).expect(INVALID_PUBLIC_KEY)
        };

        let run = tokio::spawn(crate::server::listen(
            80, context, main_http, discord_clients, #[cfg(feature = "http-interactions")] public_key
        ));
        threads.push(run);
    }

    for thread in threads {
        // When threads panic runtime shows reasons in stdout so unwrap would only make output less readable
        thread.await.unwrap_or_else(|_| std::process::exit(1));
    }
}