use std::str::FromStr;
use dotenv::dotenv;
use twilight_model::guild::Guild;
use twilight_model::id::Id;
use database::mongodb::MongoDBConnection;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB from .env");

    let connection = MongoDBConnection::connect(mongodb_url).await.unwrap();
    println!("Connected to mongodb-configs database");

    let guild_id = Id::<Guild>::from_str("898986393177567242").unwrap();

    for _ in 0..10 {
        println!("{:?}", connection.get_config(guild_id.clone()).await.unwrap());
    }

}