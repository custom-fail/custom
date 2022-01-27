use database::redis::RedisConnection;
use dotenv::dotenv;

fn main() {

    dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

    let connection = RedisConnection::connect(redis_url).unwrap();
    println!("Connected to redis database");

    println!("{:?}", connection.get_all("top_day.898986393177567242".to_string(), 5).unwrap())

}
