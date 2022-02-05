pub mod on_kick {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use chrono::Utc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::MemberRemove;
    use twilight_model::guild::audit_log::AuditLogEventType;
    use twilight_model::id::Id;
    use twilight_model::id::marker::{GenericMarker, GuildMarker};
    use twilight_util::snowflake::Snowflake;

    pub async fn run(
        event: MemberRemove,
        mongodb: MongoDBConnection,
        discord_http: Arc<Client>,
    ) -> Result<(), ()> {

        let event_target_id = event.user.id;
        let event_at = Utc::now().timestamp();

        let audit_log = discord_http
            .audit_log(event.guild_id)
            .action_type(AuditLogEventType::MemberKick)
            .limit(1).map_err(|_| ())?
            .exec().await.map_err(|_| ())?.model().await.map_err(|_| ())?;

        println!("{:?}", audit_log);

        let kick = audit_log.entries.first().ok_or(())?;
        let target_id = kick.target_id.ok_or(())?;
        if target_id.to_string() != event_target_id.to_string() {
            return Err(())
        }

        let created_at = kick.id.timestamp();
        let ping = created_at - event_at * 1000;

        if ping > 2000 {
            return Err(());
        }

        Ok(())

        // rust fmt sucks
    }
}

pub mod on_ban {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::BanAdd;

    pub async fn run(event: BanAdd, mongodb: MongoDBConnection, discord_http: Arc<Client>) {}
}

pub mod on_timeout {
    use database::mongodb::MongoDBConnection;
    use std::sync::Arc;
    use twilight_http::Client;
    use twilight_model::gateway::payload::incoming::MemberUpdate;

    pub async fn run(event: Box<MemberUpdate>, mongodb: MongoDBConnection, discord_http: Arc<Client>) {}
}
