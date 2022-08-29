use std::sync::Arc;
use futures_util::StreamExt;
use twilight_gateway::Shard;
use twilight_model::gateway::Intents;
use crate::{Bucket, MongoDBConnection, RedisConnection, ScamLinks};
use crate::events::on_event;
use twilight_http::Client;

pub async fn create_shard(
    (id, http): (String, Arc<Client>),
    mongodb: MongoDBConnection,
    redis: RedisConnection,
    scam_domains: ScamLinks,
    bucket: Bucket
) {
    let token = if let Some(token) = http.token() { token.to_string() }
    else { eprintln!("Cannot get token of client {id}"); return };

    let (shard, mut events) = Shard::new(token, Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_BANS | Intents::GUILD_MEMBERS);
    shard.start().await.expect("Failed to start shard");
    println!("Created shard for {}", id);

    while let Some(event) = events.next().await {
        tokio::spawn(on_event(
            event,
            mongodb.to_owned(),
            redis.to_owned(),
            http.to_owned(),
            scam_domains.to_owned(),
            bucket.to_owned()
        ));
    }
}