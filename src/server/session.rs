use std::collections::HashMap;
use std::sync::Arc;
use rusty_paseto::core::{ImplicitAssertion, Key, Local, PasetoSymmetricKey, V4};
use rusty_paseto::generic::GenericBuilderError;
use rusty_paseto::prelude::{PasetoBuilder, PasetoParser};
use tokio::sync::RwLock;
use twilight_http::Client;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::user::CurrentUser;
use twilight_model::util::Timestamp;
use twilight_util::snowflake::Snowflake;
use warp::Filter;
use crate::server::error::Rejection;
use crate::with_value;

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
            .set_no_expiration_danger_acknowledged()
            .build(&self.key);

        result.map(|token|
            token.strip_prefix("v4.local.").unwrap_or_default().to_string()
        )
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

pub fn authorize_user(
    authenticator: Arc<Authenticator>,
    sessions: Arc<Sessions>
) -> impl Filter<Extract = (Arc<AuthorizationInformation>,), Error = warp::Rejection> + Clone {
    let with_authenticator = with_value!(authenticator);
    let with_sessions = with_value!(sessions);

    warp::any()
        .and(warp::header("Authorization"))
        .and(warp::header("User-Id"))
        .and(with_authenticator)
        .and(with_sessions)
        .and_then(filter)
}

async fn filter(
    token: String,
    user_id: Id<UserMarker>,
    authenticator: Arc<Authenticator>,
    sessions: Arc<Sessions>
) -> Result<Arc<AuthorizationInformation>, warp::Rejection> {
    if authenticator.verify_token(token.as_str(), user_id) {
        return err!(Rejection::Unauthorized)
    }

    sessions.user(&user_id)
        .await
        .ok_or_else(|| reject!(Rejection::Unauthorized))
}