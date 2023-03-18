use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use twilight_http::Client;
use twilight_model::channel::Message;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, UserMarker, ChannelMarker};
use crate::events::automod::actions::run_action_bucket;
use crate::models::config::GuildConfig;
use crate::models::config::automod::actions::{IncreaseBucket, IncreaseBucketAmount};

pub type Bucket = Arc<Mutex<HashMap<BucketLocationKey, HashMap<String, u8>>>>;

type BucketLocationKey = [u8; 24];

// concat_bytes!() is unstable
fn create_key(
    guild_id: Id<GuildMarker>,
    channel_id: Option<Id<ChannelMarker>>,
    user_id: Id<UserMarker>
) -> BucketLocationKey {
    let b = [guild_id.get(), user_id.get(), channel_id.map(Id::get).unwrap_or(0)].map(u64::to_ne_bytes);
    [
        b[0][0], b[0][1], b[0][2], b[0][3], b[0][4], b[0][5], b[0][6], b[0][7],
        b[1][0], b[1][1], b[1][2], b[1][3], b[1][4], b[1][5], b[1][6], b[1][7],
        b[2][0], b[2][1], b[2][2], b[2][3], b[2][4], b[2][5], b[2][6], b[2][7],
    ]
}

pub async fn incr(
    discord_http: Arc<Client>,
    message: Arc<Message>,
    guild_config: Arc<GuildConfig>,
    bucket: Bucket,
    data: IncreaseBucket
) {
    let guild_id = guild_config.guild_id;
    let user_id = message.author.id;

    // TODO: replace u8 with overflow safe methods
    let amount = match data.amount {
        IncreaseBucketAmount::Stickers => message.sticker_items.len() as u8,
        IncreaseBucketAmount::Attachments => message.attachments.len() as u8,
        IncreaseBucketAmount::Mentions => message.mentions.len() as u8,
        IncreaseBucketAmount::Static(value) => value,
    };

    let bucket_data = match guild_config.get_bucket_action(&data.key) {
        Some(data) => data,
        None => return
    };

    let channel_id = data.per_channel.then_some(message.channel_id);
    let key = create_key(guild_id, channel_id, user_id);

    let count = update(&bucket, key, data.key.to_owned(), move |count| {
        if let Some(value) = count.checked_add(amount) {
            *count = value
        }
    }).await;

    // TODO: implement async actions
    if count > bucket_data.limit {
        for action in bucket_data.actions {
            run_action_bucket(
                action.action,
                message.to_owned(),
                discord_http.to_owned(),
                guild_config.to_owned(),
                bucket_data.reason.to_owned()
            ).await.ok();
        }
    }

    tokio::time::sleep(Duration::from_secs(data.duration as u64)).await;

    update(&bucket, key, data.key, move |count| {
        if let Some(value) = count.checked_sub(amount) {
            *count = value
        }
    }).await;

}

async fn update<T>(
    bucket: &Bucket,
    key: BucketLocationKey,
    name: String,
    f: T
) -> u8 where T: Fn(&mut u8) {
    let mut bucket = bucket.lock().await;

    let count = bucket
        .entry(key).or_insert_with(HashMap::new)
        .entry(name).or_insert(0);

    f(count);

    count.to_owned()
}
