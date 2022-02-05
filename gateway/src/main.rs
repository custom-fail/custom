mod events;
mod modules;

use std::path::Iter;
use crate::events::on_event;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use dotenv::dotenv;
use futures_util::StreamExt;
use std::sync::Arc;
use twilight_gateway::Shard;
use twilight_model::gateway::Intents;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
    let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

    let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
    let redis = RedisConnection::connect(redis_url).unwrap();

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("Cannot load DISCORD_TOKEN from .env");
    let discord_http = Arc::new(twilight_http::Client::new(discord_token.clone()));

    let intents = Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_BANS | Intents::GUILD_MEMBERS;
    let (shard, mut events) = Shard::new(discord_token, intents);

    shard.start().await.expect("Failed to start shard");
    println!("Created shard");

    while let Some(event) = events.next().await {
        tokio::spawn(on_event(
            event,
            mongodb.clone(),
            redis.clone(),
            discord_http.clone(),
        ));
    }
}
