use std::convert::Infallible;
use ed25519_dalek::PublicKey;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use twilight_model::application::interaction::Interaction;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use crate::authorize::verify_signature;

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
    pub fn into_response(&self) -> Result<Response<Body>, Infallible> {
        let mut response = Response::default();
        *response.status_mut() = self.status;
        *response.body_mut() = Body::from(self.body.clone());
        Ok(response)
    }
}

const METHOD_NOT_ALLOWED: HttpResponse = HttpResponse { body: "Method not allowed", status: StatusCode::METHOD_NOT_ALLOWED };
const MISSING_HEADERS: HttpResponse = HttpResponse { body:"Missing headers", status: StatusCode::BAD_REQUEST };
const UNAUTHORIZED: HttpResponse = HttpResponse { body: "Unauthorized", status: StatusCode::UNAUTHORIZED };
const INVALID_BODY: HttpResponse = HttpResponse { body: "Invalid/Missing body", status: StatusCode::BAD_REQUEST };

async fn route(request: Request<Body>, public_key: PublicKey, _mongodb: MongoDBConnection, _redis: RedisConnection) -> Result<Response<Body>, Infallible> {

    if request.method() != &Method::POST {
        return METHOD_NOT_ALLOWED.into_response();
    };

    let timestamp = request.headers().get("X-Signature-Timestamp");
    let signature = request.headers().get("X-Signature-Ed25519");

    let timestamp = match string_from_headers_option(timestamp) {
        Some(timestamp) => timestamp,
        None => return MISSING_HEADERS.into_response()
    };

    let signature = match string_from_headers_option(signature) {
        Some(signature) => signature,
        None => return MISSING_HEADERS.into_response()
    };

    let whole_body = hyper::body::to_bytes(request.into_body()).await;
    let whole_body = match whole_body {
        Ok(whole_body) => whole_body,
        Err(_) => return INVALID_BODY.into_response()
    };

    let reversed_body = whole_body.iter().cloned().collect::<Vec<u8>>();
    let body = String::from_utf8(reversed_body);
    let body = match body {
        Ok(body) => body,
        Err(_) => return INVALID_BODY.into_response()
    };

    if !verify_signature(public_key.clone(), signature, timestamp, body.clone()) {
        return UNAUTHORIZED.into_response();
    };

    let interaction = match serde_json::from_str::<Interaction>(body.as_str()) {
        Ok(value) => value,
        Err(_) => return INVALID_BODY.into_response()
    };

    Ok(Response::new(Body::from(format!("{:?}", body))))

}

pub async fn listen(port: u8, public_key: PublicKey, mongodb: MongoDBConnection, redis: RedisConnection) -> () {

    let service = make_service_fn(move |_| {
        let public_key = public_key.clone();
        let mongodb = mongodb.clone();
        let redis = redis.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                route(req, public_key.clone(), mongodb.clone(), redis.clone())
            }))
        }
    });

    let address = ([127, 0, 0, 1], port.into()).into();
    let server = hyper::Server::bind(&address).serve(service);

    println!("Listening on {}", address);

    server.await.unwrap()

}