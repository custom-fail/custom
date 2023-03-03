use std::sync::Arc;
use dashmap::DashMap;
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use twilight_http::Client;
use crate::context::Context;
use crate::MongoDBConnection;
use crate::utils::errors::Error;
use crate::gateway::shard::connect_shards;

pub type DiscordClients = Arc<DashMap<Id<ApplicationMarker>, Arc<Client>>>;

#[async_trait]
pub trait LoadDiscordClients {
    async fn load(
        mongodb: &MongoDBConnection
    ) -> Result<DiscordClients, Error>;

    fn start(
        &self,
        context: Arc<Context>
    );
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientData {
    pub application_id: Id<ApplicationMarker>,
    pub token: String,
}

#[async_trait]
impl LoadDiscordClients for DiscordClients {
    async fn load(
        mongodb: &MongoDBConnection
    ) -> Result<Self, Error> {
        let clients_data = mongodb.clients.find(doc! {}, None)
            .await.map_err(Error::from)?;
        let clients_data: Vec<ClientData> = clients_data.try_collect().await.map_err(Error::from)?;

        let clients = clients_data.iter()
            .map(|client| {
                let client = client.to_owned();
                (client.application_id, Arc::new(Client::new(client.token)))
            }).collect::<Vec<(Id<ApplicationMarker>, Arc<Client>)>>();

        Ok(Arc::new(DashMap::from_iter(clients)))
    }

    fn start(
        &self,
        context: Arc<Context>
    ) {
        for value in self.iter() {
            tokio::spawn(connect_shards(
                (value.key().to_string(), value.to_owned()),
                context.to_owned()
            ));
        }
    }
}
