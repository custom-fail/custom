use std::sync::Arc;
use dashmap::DashMap;
use futures::TryStreamExt;
use mongodb::bson::doc;
use twilight_model::id::Id;
use twilight_model::id::marker::ApplicationMarker;
use utils::errors::Error;
use crate::mongodb::MongoDBConnection;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;

pub type DiscordClients = Arc<DashMap<Id<ApplicationMarker>, Client>>;

#[async_trait]
pub trait LoadDiscordClients {
    async fn load(
        mongodb: &MongoDBConnection,
        main_client_token: Option<String>,
        main_public_key: Option<String>
    ) -> Result<DiscordClients, Error>;
}

#[derive(Clone)]
pub struct Client {
    pub public_key: String,
    pub http: Arc<twilight_http::Client>,
    pub token: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientData {
    pub application_id: Id<ApplicationMarker>,
    pub public_key: String,
    pub token: String,
}

#[async_trait]
impl LoadDiscordClients for DiscordClients {
    async fn load(
        mongodb: &MongoDBConnection,
        main_client_token: Option<String>,
        main_public_key: Option<String>
    ) -> Result<Self, Error> {
        let clients_data = mongodb.clients.find(doc! {}, None)
            .await.map_err(Error::from)?;
        let clients_data: Vec<ClientData> = clients_data.try_collect().await.map_err(Error::from)?;

        let mut clients = clients_data.iter()
            .map(|client| {
                let client = client.to_owned();
                (client.application_id, Client {
                    public_key: client.public_key,
                    token: client.token.to_owned(),
                    http: Arc::new(twilight_http::Client::new(client.token))
                })
            }).collect::<Vec<(Id<ApplicationMarker>, Client)>>();

        if let Some(main_client_token) = main_client_token {
            if let Some(main_public_key) = main_public_key {

                let discord_http = twilight_http::Client::new(main_client_token.to_owned());
                let user = discord_http.current_user()
                    .exec().await.expect("Error while loading bot data")
                    .model().await.expect("Error while loading bot data");

                clients.push((user.id.cast(), Client {
                    public_key: main_public_key,
                    http: Arc::new(discord_http),
                    token: main_client_token
                }));

            }
        }



        Ok(Arc::new(DashMap::from_iter(clients)))
    }
}
