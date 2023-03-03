/// Gives mute role after rejoin
pub mod mutes {
    use std::sync::Arc;
    use mongodb::bson::doc;
    use twilight_model::gateway::payload::incoming::MemberAdd;
    use crate::context::Context;

    pub async fn run(
        member: Box<MemberAdd>,
        discord_http: Arc<twilight_http::Client>,
        context: Arc<Context>
    ) -> Result<(), ()> {
        let config = context.mongodb.get_config(member.guild_id).await.map_err(|_| ())?;
        let mute_role = config.moderation.mute_role.ok_or(())?;

        let task = context.mongodb.tasks.find_one(doc! {
            "action": { "RemoveMuteRole": member.user.id.to_string() },
            "guild_id": config.guild_id.to_string()
        }, None).await.map_err(|_| ())?;

        if task.is_none() { return Ok(()) }

        let mut roles = member.roles.to_owned();
        roles.push(mute_role);

        discord_http.update_guild_member(config.guild_id, member.user.id)
            .roles(&roles).exec().await.map_err(|_| ())?;
        Ok(())
    }
}

/// Remove task after unban
pub mod bans {
    use mongodb::bson::doc;
    use twilight_model::gateway::payload::incoming::BanRemove;
    use crate::MongoDBConnection;

    pub async fn run(event: BanRemove, mongodb: &MongoDBConnection) -> Result<(), ()> {
        mongodb.tasks.delete_one(doc! {
            "action": { "RemoveBan": event.user.id.to_string() },
            "guild_id": event.guild_id.to_string()
        }, None).await.map_err(|_| ())?;
        Ok(())
    }
}