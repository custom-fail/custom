use std::convert::Infallible;
use std::sync::Arc;
use ed25519_dalek::PublicKey;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use twilight_http::Client;
use twilight_model::application::interaction::Interaction;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::Application;
use crate::authorize::verify_signature;
use crate::interaction::handle_interaction;

fn string_from_headers_option(header: Option<&HeaderValue>) -> Option<String> {
    Some(match header {
        Some(header) => match header.to_str() {
            Ok(header) => header.to_string(),
            Err(_) => return None
        },
        None => return None
    })
}

struct HttpResponse {
    status: StatusCode,
    body: &'static str
}

impl HttpResponse {
    pub fn to_response(&self) -> Result<Response<Body>, Infallible> {
        let mut response = Response::default();
        *response.status_mut() = self.status;
        *response.body_mut() = Body::from(self.body);
        Ok(response)
    }
}

const INTERNAL_SERVER_ERROR: HttpResponse = HttpResponse { body: "Internal server error", status: StatusCode::INTERNAL_SERVER_ERROR };
const METHOD_NOT_ALLOWED: HttpResponse = HttpResponse { body: "Method not allowed", status: StatusCode::METHOD_NOT_ALLOWED };
const MISSING_HEADERS: HttpResponse = HttpResponse { body:"Missing headers", status: StatusCode::BAD_REQUEST };
const UNAUTHORIZED: HttpResponse = HttpResponse { body: "Unauthorized", status: StatusCode::UNAUTHORIZED };
const INVALID_BODY: HttpResponse = HttpResponse { body: "Invalid/Missing body", status: StatusCode::BAD_REQUEST };

async fn route(request: Request<Body>, public_key: PublicKey, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) -> Result<Response<Body>, Infallible> {

    if request.method() != Method::POST {
        return METHOD_NOT_ALLOWED.to_response();
    };

    let timestamp = request.headers().get("X-Signature-Timestamp");
    let signature = request.headers().get("X-Signature-Ed25519");

    let timestamp = match string_from_headers_option(timestamp) {
        Some(timestamp) => timestamp,
        None => return MISSING_HEADERS.to_response()
    };

    let signature = match string_from_headers_option(signature) {
        Some(signature) => signature,
        None => return MISSING_HEADERS.to_response()
    };

    let whole_body = hyper::body::to_bytes(request.into_body()).await;
    let whole_body = match whole_body {
        Ok(whole_body) => whole_body,
        Err(_) => return INVALID_BODY.to_response()
    };

    let reversed_body = whole_body.iter().cloned().collect::<Vec<u8>>();
    let body = String::from_utf8(reversed_body);
    let body = match body {
        Ok(body) => body,
        Err(_) => return INVALID_BODY.to_response()
    };

    if !verify_signature(public_key, signature, timestamp, body.clone()) {
        return UNAUTHORIZED.to_response();
    };

    let interaction = match serde_json::from_str::<Interaction>(body.as_str()) {
        Ok(value) => value,
        Err(_) => return INVALID_BODY.to_response()
    };

    let content = handle_interaction(interaction, application, mongodb, redis, discord_http).await;
    let content = json!(content).to_string();

    let response = Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(content));

    match response {
        Ok(response) => Ok(response),
        Err(_) => INTERNAL_SERVER_ERROR.to_response()
    }

}

pub async fn listen(port: u8, public_key: PublicKey, application: Application, mongodb: MongoDBConnection, redis: RedisConnection, discord_http: Arc<Client>) {

    let service = make_service_fn(move |_| {
        let application = application.clone();
        let mongodb = mongodb.clone();
        let redis = redis.clone();
        let discord_http = discord_http.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                route(req, public_key.to_owned(), application.to_owned(), mongodb.to_owned(), redis.to_owned(), discord_http.to_owned())
            }))
        }
    });

    let address = ([127, 0, 0, 1], port.into()).into();
    let server = hyper::Server::bind(&address).serve(service);

    println!("Listening on {address}");

    server.await.unwrap()

}