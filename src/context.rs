use crate::{database::{mongodb::MongoDBConnection, redis::RedisConnection}, links::ScamLinks, bucket::Bucket, application::Application, assets::AssetsManager};

pub struct Context {
    pub application: Application,
    pub mongodb: MongoDBConnection,
    pub redis: RedisConnection,
    pub scam_domains: ScamLinks,
    pub bucket: Bucket,
    pub assets: AssetsManager
}

impl Context {
    pub async fn new() -> Self {
        let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
        let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

        let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
        let redis = RedisConnection::connect(redis_url).unwrap();

        let scam_domains = ScamLinks::new().await.expect("Cannot load scam links manager");
        scam_domains.connect();
        let bucket: Bucket = Default::default();
        let application = Application::new();
        let assets = AssetsManager::new().await;

        Self { mongodb, redis, scam_domains, bucket, application, assets }
    }
}