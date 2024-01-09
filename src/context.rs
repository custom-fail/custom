use crate::{all_macro, application::Application, database::{mongodb::MongoDBConnection, redis::RedisConnection}};

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
}

impl Context {
    pub async fn new() -> Self {
        let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
        let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

        let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
        let redis = RedisConnection::connect(redis_url).unwrap();

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
        }
    }
}
