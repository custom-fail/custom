use std::convert::Infallible;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;

fn response(text: &'static str, status: StatusCode) -> Response<Body> {
    let mut not_found = Response::default();
    *not_found.status_mut() = status;
    *not_found.body_mut() = Body::from(text);
    not_found
}

fn string_from_headers_option(header: Option<&HeaderValue>) -> Option<String> {
    Some(match header {
        Some(header) => match header.to_str() {
            Ok(header) => header.to_string(),
            Err(_) => return None
        },
        None => return None
    })
}

async fn route(request: Request<Body>, _mongodb: MongoDBConnection, _redis: RedisConnection) -> Result<Response<Body>, Infallible> {

    let method_not_allowed = Ok(response("Method not allowed", StatusCode::METHOD_NOT_ALLOWED));
    let missing_headers = Ok(response("Missing headers", StatusCode::BAD_REQUEST));

    if request.method() != &Method::POST {
        return method_not_allowed;
    };

    let timestamp = request.headers().get("X-Signature-Timestamp");
    let signature = request.headers().get("X-Signature-Ed25519");

    let timestamp = string_from_headers_option(timestamp).ok_or(missing_headers.clone())?;
    let signature = string_from_headers_option(signature).ok_or(missing_headers.clone())?;

    println!("{}, {}", timestamp, signature);

    let body = hyper::body::to_bytes(request.into_body()).await;

    Ok(Response::new(Body::from(format!("{:?}", body))))

}

pub async fn listen(port: u8, mongodb: MongoDBConnection, redis: RedisConnection) -> () {

    let service = make_service_fn(move |_| {
        let mongodb = mongodb.clone();
        let redis = redis.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                route(req, mongodb.clone(), redis.clone())
            }))
        }
    });

    let address = ([127, 0, 0, 1], port.into()).into();
    let server = hyper::Server::bind(&address).serve(service);

    println!("Listening on {}", address);

    server.await.unwrap()

}