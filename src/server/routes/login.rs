use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use twilight_http::Client;
use twilight_model::user::CurrentUser;
use twilight_model::util::Timestamp;
use warp::{Filter, Reply};
use warp::http::StatusCode;
use crate::{env_unwrap, reject, response_type, with_value};
use crate::server::error::{Rejection, MapErrorIntoInternalRejection};
use crate::server::session::{Authenticator, AuthorizationInformation, Sessions};

#[derive(Deserialize)]
struct Query {
    code: String
}

pub fn login(authenticator: Arc<Authenticator>, sessions: Arc<Sessions>) -> response_type!() {
    let with_authenticator = with_value!(authenticator);
    let with_sessions = with_value!(sessions);

    let redirect_uri = env_unwrap!("REDIRECT_URI");
    let client_secret = env_unwrap!("CLIENT_SECRET");
    let client_id = env_unwrap!("CLIENT_ID");

    let with_redirect_uri = with_value!(redirect_uri);
    let with_client_secret = with_value!(client_secret);
    let with_client_id = with_value!(client_id);

    warp::get()
        .and(warp::path("login"))
        .and(warp::query::<Query>())
        .and(with_authenticator)
        .and(with_sessions)
        .and(with_redirect_uri)
        .and(with_client_secret)
        .and(with_client_id)
        .and_then(run)
}

#[derive(Serialize)]
#[serde(tag = "grant_type")]
enum GrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode { code: String },
    #[allow(dead_code)]
    #[serde(rename = "refresh_token")]
    RefreshToken { refresh_token: String }
}

#[derive(Serialize)]
struct Data<'a> {
    pub client_id: String,
    pub client_secret: String,
    #[serde(flatten)]
    pub grant_type: GrantType,
    pub redirect_uri: Option<String>,
    pub scope: Option<&'a str>,
}

#[derive(Deserialize)]
struct PartialAuthorizationInformation {
    pub access_token: Box<str>,
    pub token_type: Box<str>,
    pub refresh_token: Box<str>,
    pub expires_in: u64,
}

const SCOPE: &str = "identify,guilds";

#[derive(Serialize)]
struct Response<'a> {
    user: &'a CurrentUser,
    token: &'a String
}

async fn run(
    query: Query,
    authenticator: Arc<Authenticator>,
    sessions: Arc<Sessions>,
    redirect_uri: String,
    client_secret: String,
    client_id: String,
) -> Result<Box<dyn Reply>, warp::Rejection> {
    let code = query.code;
    let response = reqwest::Client::new()
        .post("https://discord.com/api/oauth2/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(
            serde_urlencoded::to_string(Data {
                client_id,
                client_secret,
                grant_type: GrantType::AuthorizationCode { code },
                redirect_uri: Some(redirect_uri),
                scope: Some(SCOPE)
            }).map_err(|err| Rejection::Internal(err.into()))?
        )
        .send()
        .await
        .map_err(|err| reject!(Rejection::Internal(err.into())))?;

    if response.status() != StatusCode::OK {
        return Ok(Box::new(warp::reply::with_status(
            response.text().await.unwrap_or_else(|_| "Discord rejected the request".to_string()),
            StatusCode::BAD_REQUEST
        )))
    }

    let response: PartialAuthorizationInformation = response.json()
        .await.map_rejection()?;

    let http = Arc::new(Client::new(
        format!("{} {}", response.token_type, response.access_token)
    ));

    let user = http.current_user().await.map_rejection()?
        .model().await.map_rejection()?;
    let token = authenticator.generate_token(user.id).map_rejection()?;

    let reply = warp::reply::json(&Response {
        user: &user,
        token: &token
    });

    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH).map_rejection()?.as_secs() + response.expires_in;

    sessions.add(Arc::new(AuthorizationInformation {
        access_token: response.access_token,
        refresh_token: response.refresh_token,
        expires: Timestamp::from_secs(expires_at as i64).map_rejection()?,
        scopes: vec![],
        user,
        http,
    })).await;


    return Ok(Box::new(reply))
}