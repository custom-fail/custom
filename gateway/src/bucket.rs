use std::collections::HashMap;
use std::ops::Add;
use std::sync::Arc;
use std::time::Duration;
use dashmap::DashMap;
use tokio::sync::Mutex;
use tokio::time::{Instant, timeout_at};
use twilight_http::Client;
use twilight_model::channel::Message;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker};
use database::models::config::automod::bucket::IncreaseBucketAmount;
use database::models::config::GuildConfig;
use crate::modules::automod::actions::run_action_bucket;

pub type Bucket = Arc<DashMap<Id<GuildMarker>, Mutex<HashMap<Id<UserMarker>, HashMap<String, usize>>>>>;

pub async fn incr(
    discord_http: Arc<Client>,
    message: Message,
    guild_config: &GuildConfig,
    bucket: Bucket,
    user_id: Id<UserMarker>,
    key: String
) {
    let guild_id = guild_config.guild_id;

    let bucket_data = match guild_config.get_bucket_action(key.to_owned()) {
        Some(data) => data,
        None => return
    };

    let amount = match bucket_data.amount {
        IncreaseBucketAmount::Stickers => message.sticker_items.len(),
        IncreaseBucketAmount::Attachments => message.attachments.len(),
        IncreaseBucketAmount::Mentions => message.mentions.len(),
        IncreaseBucketAmount::Static(value) => value as usize
    };

    let guild_buckets_mutex = bucket.entry(guild_id)
        .or_insert_with(Default::default);
    let mut guild_buckets = guild_buckets_mutex.lock().await;

    let user_buckets = guild_buckets
        .entry(user_id).or_insert_with(HashMap::new);

    let count = user_buckets.entry(key.to_owned()).or_insert(0);
    *count += amount;

    if count > &mut (bucket_data.min as usize) {

        for action in bucket_data.actions {
            run_action_bucket(
                action,
                message.to_owned(),
                discord_http.to_owned(),
                guild_config,
                bucket_data.reason.to_owned()
            ).await.ok();
        }

    }

    tokio::spawn(
        timeout_at(
            Instant::now().add(Duration::from_secs(1)),
            decr(bucket.to_owned(), guild_id, user_id, amount, key)
        )
    );

}

async fn decr(bucket: Bucket, guild_id: Id<GuildMarker>, user_id: Id<UserMarker>, amount: usize, key: String) {

    let guild_buckets_mutex = bucket.entry(guild_id)
        .or_insert_with(Default::default);
    let mut guild_buckets = guild_buckets_mutex.lock().await;

    let user_buckets = guild_buckets
        .entry(user_id).or_insert_with(HashMap::new);

    let count = user_buckets.entry(key.to_owned()).or_insert(0);
    *count -= amount;

}