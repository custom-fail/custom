use std::sync::Arc;
use futures_util::StreamExt;
use twilight_gateway::stream::ShardEventStream;
use twilight_gateway::{Config, stream};
use twilight_model::gateway::Intents;
use crate::context::Context;
use crate::events::on_event;
use twilight_http::Client;

pub async fn create_shard(
    (id, http): (String, Arc<Client>),
    context: Arc<Context>
) {
    let token = if let Some(token) = http.token() { token.to_string() }
    else { eprintln!("Cannot get token of client {id}"); return };

    let intents = Intents::MESSAGE_CONTENT | Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::GUILD_MODERATION | Intents::GUILD_MEMBERS;

    let config = Config::new(token, intents);

    let mut shards = stream::create_recommended(&http, config, |_, builder| builder.build())
        .await.unwrap().collect::<Vec<_>>();

    let mut stream = ShardEventStream::new(shards.iter_mut());

    while let Some((shard, event)) = stream.next().await {
        let event = match event {
            Ok(event) => event,
            Err(err) => {
                eprintln!("error while reciving events on shard {shard} with {id} client\n{err}", shard = shard.id());
                if err.is_fatal() { break }
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