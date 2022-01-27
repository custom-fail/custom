use std::str::FromStr;
use dotenv::dotenv;
use twilight_model::guild::Guild;
use twilight_model::id::Id;
use twilight_model::user::User;
use database::mongodb::MongoDBConnection;
use mongodb::bson::doc;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");

    let connection = MongoDBConnection::connect(mongodb_url).await.unwrap();
    println!("Connected to mongodb database");

    let guild_id = Id::<Guild>::from_str("898986393177567242").unwrap();
    let user_id = Id::<User>::from_str("494386855974928386").unwrap();

    for _ in 0..10 {
        println!("{:?}", connection.get_config(guild_id.clone()).await.unwrap());
    }

    let guild_id = guild_id.to_string();
    let member_id = user_id.to_string();

    let response = connection.cases.find_one(
        doc! { "member_id": member_id, "guild_id": guild_id },
        None).await.unwrap();
    println!("{:?}", response);

}