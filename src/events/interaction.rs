use std::sync::Arc;

use twilight_http::Client;
use twilight_model::gateway::payload::incoming::InteractionCreate;

use crate::context::Context;

pub async fn run(
        interaction: Box<InteractionCreate>, discord_http: Arc<Client>, context: Arc<Context>
) -> Result<(), ()> {
    let interaction = interaction.as_ref().0.to_owned();
    println!("{interaction:?}");
    let application_id = interaction.application_id.clone();
    let interaction_id = interaction.id.cast();
    let token = interaction.token.to_owned();

    let response = crate::server::interaction::handle_interaction(
            interaction, discord_http.to_owned(), context
    ).await;

    discord_http.interaction(application_id).create_response(
            interaction_id, &token, &response
    ).await.map_err(|_| ())?;

    Ok(())
}