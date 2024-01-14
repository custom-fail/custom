use std::collections::HashMap;
use std::sync::Arc;
use rusty_paseto::core::{ImplicitAssertion, Key, Local, PasetoSymmetricKey, V4};
use rusty_paseto::generic::GenericBuilderError;
use rusty_paseto::prelude::{PasetoBuilder, PasetoParser};
use tokio::sync::RwLock;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::oauth::Application;
use twilight_model::user::{CurrentUser, User};
use twilight_model::util::Timestamp;
use twilight_util::snowflake::Snowflake;

pub struct Authenticator {
    key: PasetoSymmetricKey<V4, Local>
}

impl Authenticator {
    pub fn with_random_key() -> Self {
        Self {
            key: PasetoSymmetricKey::<V4, Local>::from(Key::try_new_random().unwrap())
        }
    }

    pub fn generate_token(&self, user_id: Id<UserMarker>) -> Result<String, GenericBuilderError> {
        let user_id = user_id.id().to_string();
        let assertion = ImplicitAssertion::from(user_id.as_str());

        let result = PasetoBuilder::<V4, Local>::default()
            .set_implicit_assertion(assertion)
            .build(&self.key);

        result
    }

    pub fn verify_token(&self, token: &str, user_id: Id<UserMarker>) -> bool {
        let user_id = user_id.to_string();
        let assertion = ImplicitAssertion::from(user_id.as_str());


        let is_verified = PasetoParser::<V4, Local>::default()
            .set_implicit_assertion(assertion)
            .parse(token, &self.key)
            .is_ok();

        is_verified
    }
}

#[derive(Clone)]
pub struct AuthorizationInformation {
    pub access_token: Box<str>,
    pub refresh_token: Box<str>,
    pub expires: Timestamp,
    pub scopes: Vec<String>,
    pub user: CurrentUser,
    pub http: Arc<Client>,
}

#[derive(Default)]
pub struct Sessions(RwLock<HashMap<Id<UserMarker>, Arc<AuthorizationInformation>>>);

impl Sessions {
    pub async fn user(&self, id: &Id<UserMarker>) -> Option<Arc<AuthorizationInformation>> {
        Some(Arc::clone(self.0.read().await.get(&id)?))
    }

    pub async fn add<'f>(&self,  data: Arc<AuthorizationInformation>) {
        self.0.write().await.insert(data.user.id, data);
    }

    pub async fn refresh(&self, user_id: &Id<UserMarker>) {
        todo!()
    }
}