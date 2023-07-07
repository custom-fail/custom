use crate::{database::{mongodb::MongoDBConnection, redis::RedisConnection}, links::ScamLinks, bucket::Bucket, application::Application, assets::AssetsManager};
use crate::utils::cli::LoadingAnimation;

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

        let loading = LoadingAnimation::new("Connecting to: MongoDB");
        let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
        loading.finish("Connected to: MongoDB ");

        let loading = LoadingAnimation::new("Connecting to: Redis");
        let redis = RedisConnection::connect(redis_url).unwrap();
        loading.finish("Connected to: Redis ");

        // loading it after scam_domains messages makes stdout writing over stdin
        let assets = AssetsManager::new().await;
        let scam_domains = ScamLinks::new()
            .await
            .expect("Cannot load scam links manager");
        scam_domains.connect();
        let bucket: Bucket = Default::default();
        let application = Application::new();

        Self { mongodb, redis, scam_domains, bucket, application, assets }
    }
}