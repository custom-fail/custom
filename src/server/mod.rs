use crate::all_macro;

#[cfg(any(
    feature = "gateway",
    feature = "custom-clients",
    feature = "http-interactions"
))]
pub mod interaction;

#[cfg(any(feature = "http-interactions", feature = "api"))]
#[macro_use]
pub mod error;

all_macro!(
    cfg(any(feature = "http-interactions", feature = "api"));
    // pub mod error;
    pub mod routes;
);

#[cfg(feature = "api")]
mod session;

#[cfg(feature = "http-interactions")]
pub mod authorize;

#[cfg(feature = "api")]
pub mod guild {
    pub mod commands;
    pub mod editing;
    pub mod ws;
}

#[cfg(any(feature = "api", feature = "http-interactions"))]
mod http_server {
    use std::env;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::str::FromStr;
    use std::sync::Arc;
    use tracing::warn;
    use twilight_http::Client;
    use warp::Filter;
    use warp::http::{HeaderName, Method};
    use crate::context::Context;

    #[macro_export]
    macro_rules! with_value {
        ($name: expr) => {
            warp::any().map(move || $name.to_owned())
        };
    }

    #[macro_export]
    macro_rules! response_type {
        () => {
            impl warp::Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
        };
    }

    pub async fn listen(
        port: u16,
        context: Arc<Context>,
        discord_http: Arc<Client>,
        #[cfg(feature = "http-interactions")] public_key: ed25519_dalek::VerifyingKey
    ) {
        let cors_allow = if let Ok(origin) = env::var("ALLOWED_ORIGIN") {
            warp::cors()
                .allow_origin(origin.as_str())
        } else {
            warn!(
                "There is no ALLOWED_ORIGIN environment variable, CORS Headers are set to accept all requests"
            );
            warp::cors().allow_any_origin()
        };
        let cors_allow = cors_allow
            .allow_headers([
                HeaderName::from_str("Authorization").unwrap(),
                HeaderName::from_str("User-Id").unwrap()
            ])
            .allow_methods([Method::GET, Method::POST])
            .allow_credentials(true)
            .build();

        let routes = crate::server::routes::get_all_routes(
            discord_http, context, #[cfg(feature = "http-interactions")] public_key
        )
            .recover(crate::server::error::handle_rejection)
            .with(cors_allow);

        const ALL_SOCKETS: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        warp::serve(routes).run(SocketAddr::new(ALL_SOCKETS, port)).await;
    }
}

#[cfg(any(feature = "api", feature = "http-interactions"))]
pub use http_server::*;