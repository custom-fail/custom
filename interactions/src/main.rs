use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use dotenv::dotenv;

#[tokio::main]
async fn main() {

    dotenv();

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
    let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

    let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
    let redis = RedisConnection::connect(redis_url).unwrap();

}