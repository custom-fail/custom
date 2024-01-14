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

mod http_server {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use twilight_http::Client;
    use warp::Filter;
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
        #[cfg(feature = "http-interactions")] public_key: ed25519_dalek::PublicKey
    ) {
        let routes = crate::server::routes::get_all_routes(
            discord_http, context, #[cfg(feature = "http-interactions")] public_key
        ).recover(crate::server::error::handle_rejection);

        const ALL_SOCKETS: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        warp::serve(routes).run(SocketAddr::new(ALL_SOCKETS, port)).await;
    }
}

#[cfg(any(feature = "api", feature = "http-interactions"))]
pub use http_server::*;