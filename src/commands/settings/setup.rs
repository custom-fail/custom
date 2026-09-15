use crate::commands::context::InteractionContext;
use crate::commands::ResponseData;
use crate::context::Context;
use crate::models::config::GuildConfig;
use crate::render_context;
use std::sync::Arc;
use twilight_http::Client;

pub async fn run(
    interaction: InteractionContext,
    context: Arc<Context>,
    _: Arc<Client>,
    config: GuildConfig,
) -> ResponseData {
    Ok((
        config
            .assets
            .render_message(
                &context.assets,
                "commands.setup",
                &mut render_context!(["interaction", &interaction.orginal]),
                &context.redis,
            )
            .await?,
        None,
    ))
}
