use std::sync::Arc;
use reqwest::StatusCode;
use twilight_http::Client;
use warp::Filter;
use crate::context::Context;
use crate::response_type;

#[cfg(feature = "http-interactions")]
mod interactions;
#[cfg(feature = "api")]
mod login;

pub fn get_all_routes(
    discord_http: Arc<Client>,
    context: Arc<Context>,
    #[cfg(feature = "http-interactions")] public_key: ed25519_dalek::PublicKey
) -> response_type!() {
    let filter = warp::path::end().map(|| {
        warp::reply::with_status("👀", StatusCode::OK)
    });

    #[cfg(feature = "http-interactions")]
    let filter = filter.or(interactions::filter(
        discord_http, context, public_key
    ));

    #[cfg(feature = "api")]
    let (sessions, authenticator) = {
        (
            Arc::new(crate::server::session::Sessions::default()),
            Arc::new(crate::server::session::Authenticator::with_random_key())
        )
    };

    #[cfg(feature = "api")]
    let filter = filter.or(login::login(authenticator, sessions));

    filter
}