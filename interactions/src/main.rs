mod server;
mod authorize;
mod commands;
mod interaction;
mod application;

use std::collections::HashMap;
use std::sync::Arc;
use database::models::config::GuildConfig;
use database::mongodb::MongoDBConnection;
use database::redis::RedisConnection;
use dotenv::dotenv;
use ed25519_dalek::PublicKey;
use futures::FutureExt;
use crate::application::{Application, Component, Modal};
use crate::commands::Command;
use twilight_http::Client;
use crate::commands::context::InteractionContext;

#[tokio::main]
async fn main() {

    dotenv().ok();

    let public_key_env = std::env::var("PUBLIC_KEY").expect("Cannot load PUBLIC_KEY from .env");
    let pbk_bytes = hex::decode(public_key_env.as_str()).expect("Invalid public value");
    let public_key = PublicKey::from_bytes(&pbk_bytes).expect("Unknown public key");

    let mongodb_url = std::env::var("MONGODB_URL").expect("Cannot load MONGODB_URL from .env");
    let redis_url = std::env::var("REDIS_URL").expect("Cannot load REDIS_URL from .env");

    let mongodb = MongoDBConnection::connect(mongodb_url).await.unwrap();
    let redis = RedisConnection::connect(redis_url).unwrap();

    let discord_token = std::env::var("DISCORD_TOKEN").expect("Cannot load DISCORD_TOKEN from .env");
    let discord_http = Arc::new(twilight_http::Client::new(discord_token));

    let application = Application::new();
    application.add_commands(vec![

        command!("case details", "moderation", crate::commands::case::details::run),
        command!("case remove", "moderation", crate::commands::case::remove::run),
        command!("case edit", "moderation", crate::commands::case::edit::run),
        command!("case last", "moderation", crate::commands::case::last::run),
        command!("case list", "moderation", crate::commands::case::list::run),

        command!("kick", "moderation", crate::commands::moderation::execute::run),
        command!("mute", "moderation", crate::commands::moderation::execute::run),
        command!("warn", "moderation", crate::commands::moderation::execute::run),
        command!("ban", "moderation", crate::commands::moderation::execute::run),

        command!("mod-dash", "moderation", crate::commands::moderation::dash::run),
        command!("clear", "moderation", crate::commands::moderation::clear::run),

        command!("top week all", "top", crate::commands::top::all::run),
        command!("top day all", "top", crate::commands::top::all::run),
        command!("top week me", "top", crate::commands::top::me::run),
        command!("top day me", "top", crate::commands::top::me::run)

    ]).await;

    application.add_components(vec![
        // page switching in case list command
        Component {
            options: vec![("member".to_string(), "User".to_string())],
            values: vec![("page".to_string(), "Integer".to_string())],
            command: "case list".to_string(),
            id: "cl".to_string()
        },
        // mod panel
        Component {
            options: vec![("action".to_string(), "String".to_string())],
            values: vec![],
            command: "mod-dash".to_string(),
            id: "mod-panel".to_string()
        }
    ]).await;

    application.add_modals(vec![
        // context menu moderation commands
        Modal {
            options: vec![("member".to_string(), "User".to_string())],
            inputs: HashMap::from([("reason".to_string(), "String".to_string())]),
            command: "warn".to_string(),
            id: "warn".to_string()
        },
        Modal {
            options: vec![("member".to_string(), "User".to_string())],
            inputs: HashMap::from([("reason".to_string(), "String".to_string())]),
            command: "kick".to_string(),
            id: "kick".to_string()
        },
        Modal {
            options: vec![("member".to_string(), "User".to_string())],
            inputs: HashMap::from([
                ("duration".to_string(), "String".to_string()),
                ("reason".to_string(), "String".to_string())
            ]),
            command: "mute".to_string(),
            id: "mute".to_string()
        },
        Modal {
            options: vec![("member".to_string(), "User".to_string())],
            inputs: HashMap::from([("reason".to_string(), "String".to_string())]),
            command: "ban".to_string(),
            id: "ban".to_string()
        },
        // mod panel
        Modal {
            options: vec![],
            inputs: HashMap::from([
                ("member".to_string(), "User".to_string()),
                ("reason".to_string(), "String".to_string())
            ]),
            command: "kick".to_string(),
            id: "kick-d".to_string()
        },
        Modal {
            options: vec![],
            inputs: HashMap::from([
                ("member".to_string(), "User".to_string()),
                ("reason".to_string(), "String".to_string())
            ]),
            command: "warn".to_string(),
            id: "warn-d".to_string()
        },
        Modal {
            options: vec![],
            inputs: HashMap::from([
                ("member".to_string(), "User".to_string()),
                ("duration".to_string(), "String".to_string()),
                ("reason".to_string(), "String".to_string())
            ]),
            command: "mute".to_string(),
            id: "mute-d".to_string()
        },
        Modal {
            options: vec![],
            inputs: HashMap::from([
                ("member".to_string(), "User".to_string()),
                ("reason".to_string(), "String".to_string())
            ]),
            command: "ban".to_string(),
            id: "ban-d".to_string()
        }
    ]).await;

    server::listen(80, public_key, application, mongodb, redis, discord_http).await;

}