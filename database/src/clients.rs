use std::sync::Arc;
use dashmap::DashMap;
use mongodb::bson::doc;
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use utils::errors::Error;
use crate::mongodb::MongoDBConnection;

pub struct DiscordClients {
    pub main_client: Arc<twilight_http::Client>,
    pub http_client: DashMap<Id<ApplicationMarker>, Arc<twilight_http::Client>>
}

pub struct ClientData {
    pub application_id: Id<ApplicationMarker>,
    pub token: String,
}

impl DiscordClients {
    pub fn load(mongodb: &MongoDBConnection, main_client_token: String) -> Self {
        let clients_data = mongodb.clients.find(doc! {}, None).await?;
        let clients_data: Vec<ClientData> = clients_data.try_collect().await.map_err(Error::from)?;

        let clients = clients_data.iter()
            .map(|client|
                (client.application_id, twilight_http::Client::new(client.token.to_owned()))
            ).collect::<(Id<ApplicationMarker>, Arc<twilight_http::Client>)>();

        Self {
            main_client: Arc::new(twilight_http::Client::new(main_client_token)),
            http_client: DashMap::from_iter(clients)
        }
    }
}
