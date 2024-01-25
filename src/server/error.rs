use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter};
use reqwest::StatusCode;
use warp::reject::Reject;
use warp::Reply;

#[derive(Debug)]
pub enum Rejection {
    #[cfg(feature = "http-interactions")]
    BodyNotConvertableToString,
    #[cfg(feature = "http-interactions")]
    InvalidSignature,
    #[cfg(feature = "api")]
    Unauthorized,
    #[cfg(feature = "api")]
    MissingPermissions,
    #[cfg(feature = "api")]
    NotMutualGuild,
    Internal(anyhow::Error)
}

impl Display for Rejection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "http-interactions")]
            Rejection::BodyNotConvertableToString => f.write_str("Cannot convert bytes from body into utf8 encoded string"),
            #[cfg(feature = "http-interactions")]
            Rejection::InvalidSignature => f.write_str("Couldn't verify signature"),
            #[cfg(feature = "api")]
            Rejection::Unauthorized => f.write_str("Invalid authorization data provided"),
            #[cfg(feature = "api")]
            Rejection::MissingPermissions => f.write_str("This account isn't authorized to perform this action"),
            #[cfg(feature = "api")]
            Rejection::NotMutualGuild => f.write_str("You can't manage this guild without adding bot first"),
            Rejection::Internal(err) => std::fmt::Display::fmt(&err, f),
        }?;
        Ok(())
    }
}

// impl warp::reject::

#[macro_export()]
macro_rules! err {
    ($err: expr) => {
        Err($crate::reject!($err))
    };
}

#[macro_export]
macro_rules! reject {
    ($err: expr) => {
        (warp::reject::custom($err))
    };
}

pub trait MapErrorIntoInternalRejection<T> {
    fn map_rejection(self) -> Result<T, warp::Rejection>;
}

impl<T, E: Into<anyhow::Error>> MapErrorIntoInternalRejection<T> for Result<T, E> where Self: Sized {
    fn map_rejection(self) -> Result<T, warp::Rejection> {
        self.map_err(|err| reject!(Rejection::Internal(err.into())))
    }
}

impl Reject for Rejection {}

pub async fn handle_rejection(rejection: warp::Rejection) -> Result<impl Reply, Infallible> {
    println!("{:?}", rejection);
    Ok(if let Some(rejection) = rejection.find::<Rejection>() {
        warp::reply::with_status(rejection.to_string(), match rejection {
            #[cfg(feature = "http-interactions")]
            Rejection::BodyNotConvertableToString => StatusCode::BAD_REQUEST,
            #[cfg(feature = "http-interactions")]
            Rejection::InvalidSignature => StatusCode::BAD_REQUEST,
            #[cfg(feature = "api")]
            Rejection::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        })
    } else {
        warp::reply::with_status(
            "Internal Server Error".to_string(), StatusCode::INTERNAL_SERVER_ERROR
        )
    })
}
