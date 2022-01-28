use std::convert::Infallible;
use hyper::{Body, Request, Response};
use hyper::service::{make_service_fn, service_fn};
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;

async fn route(_: Request<Body>, _mongodb: MongoDBConnection, _redis: RedisConnection) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
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