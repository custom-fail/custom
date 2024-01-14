use std::sync::Arc;
use ed25519_dalek::PublicKey;
use twilight_http::Client;
use twilight_model::application::interaction::Interaction;
use warp::Filter;
use warp::reply::Json;
use crate::context::Context;
use crate::server::interaction::handle_interaction;
use crate::{response_type, with_value};

pub fn filter(
    discord_http: Arc<Client>,
    context: Arc<Context>,
    public_key: PublicKey
) -> response_type!() {
    let with_discord_http = with_value!(discord_http);
    let with_context = with_value!(context);

    warp::post()
        .and(warp::path("interactions"))
        .and(crate::server::authorize::filter(public_key))
        .and(warp::body::json())
        .and(with_discord_http)
        .and(with_context)
        .and_then(run)
}

async fn run(content: Interaction, discord_http: Arc<Client>, context: Arc<Context>) -> Result<Json, warp::Rejection> {
    let response = handle_interaction(
        content,
        discord_http,
        context
    ).await;

    Ok(warp::reply::json(&response))
}