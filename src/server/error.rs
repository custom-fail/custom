use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter, Pointer, Write};
use reqwest::StatusCode;
use warp::reject::Reject;
use warp::Reply;

#[derive(Debug)]
pub enum Rejection {
    BodyNotConvertableToString,
    InvalidSignature,
    InvalidCode,
    Internal(anyhow::Error)
}

impl Display for Rejection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Rejection::BodyNotConvertableToString => f.write_str("Cannot convert bytes from body into utf8 encoded string"),
            Rejection::InvalidSignature => f.write_str("Couldn't verify signature"),
            Rejection::InvalidCode => f.write_str("Invalid `code` was provided"),
            Rejection::Internal(err) => std::fmt::Display::fmt(&err, f)
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

// impl Reply for Rejection {
//     fn into_response(self) -> Response {
//         handle_rejection(self)
//     }
// }

pub async fn handle_rejection(rejection: warp::Rejection) -> Result<impl Reply, Infallible> {
    Ok(if let Some(rejection) = rejection.find::<Rejection>() {
        warp::reply::with_status(rejection.to_string(), match rejection {
            Rejection::BodyNotConvertableToString => StatusCode::BAD_REQUEST,
            Rejection::InvalidSignature => StatusCode::BAD_REQUEST,
            Rejection::InvalidCode => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR
        })
    } else {
        warp::reply::with_status(
            "Internal Server Error".to_string(), StatusCode::INTERNAL_SERVER_ERROR
        )
    })
}
