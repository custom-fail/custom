use std::sync::Arc;
use crate::context::Context;
use dotenv::dotenv;
use tokio::task::JoinHandle;
use twilight_http::Client;

all_macro!(
    cfg(feature = "gateway");
    mod events;
    mod links;
    mod bucket;
);

mod context;

#[cfg(any(feature = "gateway", feature = "custom-clients", feature = "tasks"))]
mod gateway;

mod server {
    use crate::all_macro;
    #[cfg(any(
        feature = "gateway",
        feature = "custom-clients",
        feature = "http-interactions"
    ))]
    pub mod interaction;

    all_macro!(
        cfg(feature = "http-interactions");
        mod authorize;
        pub mod run;
    );
}

#[cfg(feature = "tasks")]
mod tasks;
mod application;
mod commands;
mod database;
mod models;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let context = Arc::new(Context::new().await);

    let discord_token = std::env::var("DISCORD_TOKEN")
        .expect("Cannot load DISCORD_TOKEN from .env");
    let main_http = Arc::new(Client::new(discord_token.to_owned()));

    let mut threads: Vec<JoinHandle<()>> = vec![];

    #[cfg(any(feature = "custom-clients", feature = "tasks"))]
    {
        use crate::gateway::clients::{DiscordClients, LoadDiscordClients};
        let discord_clients = DiscordClients::load(&context.mongodb).await.unwrap();

        #[cfg(feature = "tasks")]
        {
            threads.push(tasks::run(
                context.mongodb.to_owned(),
                discord_clients.to_owned(),
                main_http.to_owned()
            ));
        }

        #[cfg(feature = "custom-clients")]
        threads.append(&mut discord_clients.start(context.to_owned()));
    }

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

    #[cfg(feature = "http-interactions")]
    {
        use ed25519_dalek::PublicKey;
        const INVALID_PUBLIC_KEY: &str = "PUBLIC_KEY provided in .env is invalid";

        let public_key = std::env::var("PUBLIC_KEY").expect(INVALID_PUBLIC_KEY);
        let pbk_bytes = hex::decode(public_key.as_str()).expect(INVALID_PUBLIC_KEY);
        let public_key = PublicKey::from_bytes(&pbk_bytes).expect(INVALID_PUBLIC_KEY);

        let run = tokio::spawn(crate::server::run::listen(
            80, context, main_http, public_key
        ));
        threads.push(run);
    }

    for thread in threads {
        thread.await.unwrap();
    }
}