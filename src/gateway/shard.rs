use std::sync::Arc;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt};
use twilight_model::gateway::{Intents, ShardId};
use crate::context::Context;
use crate::events::on_event;
use twilight_http::Client;

pub async fn connect_shards(
    (id, http): (String, Arc<Client>),
    context: Arc<Context>
) {
    let token = if let Some(token) = http.token() { token.to_string() }
    else { eprintln!("Cannot get token of client {id}"); return };

    let intents = Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_MODERATION | Intents::GUILD_MEMBERS;

    let mut shard = Shard::new(ShardId::ONE, token, intents);

    while let Some(event) = shard.next_event(EventTypeFlags::all()).await {
        let event = match event {
            Ok(event) => event,
            Err(err) => {
                eprintln!(
                    "error while receiving events on shard {shard} with {id} client\n{err}",
                    shard = shard.id().number()
                );
                continue;
            }
        };

        tokio::spawn(on_event(
            event,
            context.to_owned(),
            http.to_owned()
        ));
    }
}