use std::convert::Infallible;
use std::fmt::Display;
use ed25519_dalek::Verifier;
use ed25519_dalek::{PublicKey, Signature};
use std::str::FromStr;
use futures_util::{StreamExt, TryFutureExt, TryStreamExt};
use warp::Filter;
use warp::hyper::body::Bytes;
use warp::path::Exact;
use warp::reject::Reject;
use crate::server::error::{MapErrorIntoInternalRejection, Rejection};
use crate::{err, reject, with_value};

pub fn verify_signature(
    public_key: PublicKey,
    signature: String,
    timestamp: String,
    body: String,
) -> bool {
    let signature = Signature::from_str(signature.as_str());
    let signature = match signature {
        Ok(signature) => signature,
        Err(_) => return false
    };

    let msg = format!("{timestamp}{body}");
    let verified = public_key.verify(msg.as_bytes(), &signature);

    verified.is_ok()
}

pub fn filter(public_key: PublicKey)
    -> impl Filter<Extract = (), Error = warp::Rejection> + Clone {
    let with_public_key = with_value!(public_key);

    warp::any()
        .and(with_public_key)
        .and(warp::header("X-Signature-Timestamp"))
        .and(warp::header("X-Signature-Ed25519"))
        .and(warp::body::bytes())
        .and_then(f)
        .untuple_one()
        .boxed()
}

async fn f(public_key: PublicKey, timestamp: String, signature: String, body: Bytes) -> Result<(), warp::Rejection> {
    let body = String::from_utf8(body.to_vec()).map_rejection()?;

    if !verify_signature(
        public_key,
        signature,
        timestamp,
        body
    ) {
        return err!(Rejection::InvalidSignature);
    }

    Ok(())
}
