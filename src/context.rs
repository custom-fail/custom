use crate::{
    all_macro,
    env_unwrap,
    application::Application,
    database::{mongodb::MongoDBConnection, redis::RedisConnection},
};

all_macro!(
    cfg(feature = "gateway");
    use crate::bucket::Bucket;
    use crate::links::ScamLinks;
);

pub struct Context {
    pub application: Application,
    pub mongodb: MongoDBConnection,
    pub redis: RedisConnection,
    #[cfg(feature = "gateway")]
    pub scam_domains: ScamLinks,
    #[cfg(feature = "gateway")]
    pub bucket: Bucket,
    pub assets: AssetsManager
}

impl Context {
    pub async fn new() -> Self {
        let mongodb_uri = env_unwrap!("MONGODB_URI");
        let redis_url = env_unwrap!("REDIS_URL");

        let loading = LoadingAnimation::new("Connecting to: MongoDB");
        let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
        loading.finish("Connected to: MongoDB ");

        let loading = LoadingAnimation::new("Connecting to: Redis");
        let redis = RedisConnection::connect(redis_url).unwrap();
        loading.finish("Connected to: Redis ");

        // loading it after scam_domains messages makes stdout writing over stdin
        let assets = AssetsManager::new().await;

        #[cfg(feature = "gateway")]
        let scam_domains = ScamLinks::new()
            .await
            .expect("Cannot load scam links manager");
        #[cfg(feature = "gateway")]
        scam_domains.connect();

        #[cfg(feature = "gateway")]
        let bucket: Bucket = Default::default();
        let application = Application::new();

        Self {
            mongodb,
            redis,
            #[cfg(feature = "gateway")]
            scam_domains,
            #[cfg(feature = "gateway")]
            bucket,
            application,
            assets,
        }
    }
}
