use std::sync::Arc;
use crate::bucket::Bucket;
use crate::database::mongodb::MongoDBConnection;
use crate::database::redis::RedisConnection;
use crate::gateway::clients::{DiscordClients, LoadDiscordClients};
use crate::gateway::shard::create_shard;
use crate::links::ScamLinks;
use dotenv::dotenv;
use ed25519_dalek::PublicKey;
use twilight_http::Client;
use crate::application::Application;

mod events;
mod links;
mod bucket;
mod server;
mod database;
mod utils;
mod commands;
mod tasks;
mod models;
mod gateway;
mod application;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let args: Vec<String> = std::env::args().collect();

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
    let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

    let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
    let redis = RedisConnection::connect(redis_url).unwrap();

    let discord_token = std::env::var("DISCORD_TOKEN")
        .expect("Cannot load DISCORD_TOKEN from .env");

    let main_http = Arc::new(Client::new(discord_token.to_owned()));

    if args.contains(&"--gateway".to_string()) || args.contains(&"-A".to_string()) {
        let scam_domains = ScamLinks::new()
            .await.expect("Cannot load scam links manager");
        scam_domains.connect();

        let bucket: Bucket = Default::default();

        if args.contains(&"--custom-clients".to_string()) || args.contains(&"-A".to_string()) {
            let discord_clients = DiscordClients::load(&mongodb).await.unwrap();

            if args.contains(&"--tasks".to_string()) || args.contains(&"A".to_string()) {
                tasks::run(
                    mongodb.to_owned(),
                    discord_clients.to_owned(),
                    main_http.to_owned()
                );
            }

            discord_clients.start(
                mongodb.to_owned(),
                redis.to_owned(),
                scam_domains.to_owned(),
                bucket.to_owned()
            );
        }

        let run = create_shard(
            ("main".to_string(), Arc::new(
                Client::new(discord_token.to_owned())
            )),
            mongodb.to_owned(),
            redis.to_owned(),
            scam_domains,
            bucket
        );

        if args.contains(&"--http".to_string()) || args.contains(&"-A".to_string()) {
            tokio::spawn(run);
        } else { run.await; }
    }


    const INVALID_PUBLIC_KEY: &str = "PUBLIC_KEY provided in .env is invalid";

    if args.contains(&"--http".to_string()) || args.contains(&"-A".to_string()) {
        let public_key = std::env::var("PUBLIC_KEY")
            .expect("Cannot load PUBLIC_KEY from .env");
        let pbk_bytes = hex::decode(public_key.as_str()).expect(INVALID_PUBLIC_KEY);
        let public_key = PublicKey::from_bytes(&pbk_bytes).expect(INVALID_PUBLIC_KEY);
        let application = Application::new();
        crate::server::server::listen(
            80, application, mongodb, redis, main_http, public_key
        ).await;
    }
}