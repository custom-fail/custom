use crate::bucket::Bucket;
use crate::context::Context;
use crate::database::mongodb::MongoDBConnection;
use crate::database::redis::RedisConnection;
use crate::gateway::clients::{DiscordClients, LoadDiscordClients};
use crate::gateway::shard::connect_shards;
use crate::links::ScamLinks;
use dotenv::dotenv;
use ed25519_dalek::PublicKey;
use std::sync::Arc;
use twilight_http::Client;

mod application;
mod assets;
mod bucket;
mod commands;
mod context;
mod database;
mod events;
mod gateway;
mod links;
mod models;
mod server;
mod tasks;
mod utils;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args: Vec<String> = std::env::args().collect();
    let context = Arc::new(Context::new().await);

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("Cannot load DISCORD_TOKEN from .env");
    let main_http = Arc::new(Client::new(discord_token.to_owned()));

    if args.contains(&"--gateway".to_string()) || args.contains(&"-A".to_string()) {
        if args.contains(&"--custom-clients".to_string()) || args.contains(&"-A".to_string()) {
            let discord_clients = DiscordClients::load(&context.mongodb).await.unwrap();

            if args.contains(&"--tasks".to_string()) || args.contains(&"A".to_string()) {
                tasks::run(
                    context.mongodb.to_owned(),
                    discord_clients.to_owned(),
                    main_http.to_owned(),
                );
            }

            discord_clients.start(context.to_owned());
        }

        let run = connect_shards(
            (
                "main".to_string(),
                Arc::new(Client::new(discord_token.to_owned())),
            ),
            context.to_owned(),
        );

        if args.contains(&"--http".to_string()) || args.contains(&"-A".to_string()) {
            tokio::spawn(run);
        } else {
            run.await;
        }
    }

    const INVALID_PUBLIC_KEY: &str = "PUBLIC_KEY provided in .env is invalid";

    if args.contains(&"--http".to_string()) || args.contains(&"-A".to_string()) {
        let public_key = std::env::var("PUBLIC_KEY").expect("Cannot load PUBLIC_KEY from .env");
        let pbk_bytes = hex::decode(public_key.as_str()).expect(INVALID_PUBLIC_KEY);
        let public_key = PublicKey::from_bytes(&pbk_bytes).expect(INVALID_PUBLIC_KEY);

        crate::server::listen(80, context, main_http, public_key).await;
    }
}
