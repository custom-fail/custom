mod server;
mod authorize;
mod interaction;
mod commands;
mod application;

use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use dotenv::dotenv;
use ed25519_dalek::PublicKey;
use crate::application::Application;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let public_key_env = std::env::var("PUBLIC_KEY").expect("Cannot load PUBLIC_KEY from .env");
    let pbk_bytes = hex::decode(public_key_env.as_str()).expect("Invalid public value");
    let public_key = PublicKey::from_bytes(&pbk_bytes.as_ref()).expect("Unknown public key");

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
    let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

    let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
    let redis = RedisConnection::connect(redis_url).unwrap();

    let mut application = Application::new();
    application.add_command(Vec::new());

    server::listen(80, public_key, application, mongodb, redis).await;

}