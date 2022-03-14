use std::sync::Arc;
use twilight_http::Client;
use twilight_model::channel::embed::{Embed, EmbedAuthor};
use twilight_model::gateway::payload::incoming::MessageCreate;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;
use twilight_model::util::ImageHash;
use database::models::config::automod::actions::Actions;
use database::models::config::automod::checks::Checks;
use database::models::config::automod::filters::Filters;
use database::mongodb::MongoDBConnection;
use crate::{ok_or_skip, ScamLinks};

fn get_avatar_from_member(avatar: Option<ImageHash>, user_id: Id<UserMarker>) -> String {
    match avatar {
        Some(avatar) => {
            let file_format = if avatar.is_animated() { "gif" } else { "png" };
            format!("https://cdn.discordapp.com/avatars/{}/{}.{}", user_id, avatar, file_format)
        }
        None =>  "https://cdn.discordapp.com/embed/avatars/0.png".to_string()
    }
}

pub async fn run(message: Box<MessageCreate>, mongodb: MongoDBConnection, discord_http: Arc<Client>, scam_domains: ScamLinks) -> Result<(), ()> {

    let invites = regex::Regex::new(r"(?i)(discord.gg|discordapp.com/invite|discord.com/invite)(?:/#)?/([a-zA-Z0-9-]+)").map_err(|_| ())?;
    let domains = regex::Regex::new(r"(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z0-9][a-z0-9-]{0,61}[a-z0-9]").map_err(|_| ())?;

    let guild_id = message.guild_id.ok_or(())?;
    let guild_config = mongodb.get_config(guild_id).await.map_err(|e| {
        println!("{e}");
        ()
    })?;

    if message.content.len() == 0 || message.author.bot {
        return Ok(())
    }

    let message_content = message.content.to_lowercase();

    for automod in guild_config.moderation.automod {

        for filter in automod.filters {
            let is_filtred = match filter {
                Filters::Attachments(config) => {
                    let message_attachments_count = message.attachments.len();
                    (if let Some(min) = config.min {
                        (min as usize) > message_attachments_count
                    } else { false }) || (if let Some(max) = config.max {
                        (max as usize) < message_attachments_count
                    } else { false })
                },
                Filters::MessageLength(config) => {
                    let message_content_len = message.content.len();
                    (if let Some(min) = config.min {
                        (min as usize) > message_content_len
                    } else { false }) || (if let Some(max) = config.max {
                        (max as usize) < message_content_len
                    } else { false })
                },
                Filters::Stickers => { message.sticker_items.len() > 0 }
            };
            println!("{is_filtred}");
            if is_filtred { return Ok(()) }
        }
        println!("dsadas");
        for check in automod.checks {

            let is_allowed = match check {
                Checks::FlaggedScamLink => {
                    let domains = domains.find_iter(message_content.as_str());
                    let domains = domains.map(|domain| domain.as_str().to_string()).collect();
                    scam_domains.contains(domains).await
                },
                Checks::TextLines(config) => {
                    let enters = message.content.lines().count();
                    let line_len = if let Some(len) = config.line_len { len as usize } else { 120 };
                    let split = message.content.len() / line_len;
                    let lines = enters + split;
                    (if let Some(min) = config.min {
                        lines < (min as usize)
                    } else { false }) || (if let Some(max) = config.max {
                        lines > (max as usize)
                    } else { false })
                },
                Checks::CapsLock(config) => {
                    let uppercase = message.content.chars().filter(|c| c.is_uppercase()).count();
                    let uppercase_part = uppercase * 100 / message.content.len();
                    (if let Some(min) = config.min {
                        uppercase_part < (min as usize)
                    } else { false }) || (if let Some(max) = config.max {
                        uppercase_part > (max as usize)
                    } else { false })
                },
                Checks::Invites(config) => {
                    let invites = invites.find_iter(message_content.as_str());
                    let mut contains = false;
                    for invite in invites {
                        let code = ok_or_skip!(invite.as_str().split("/").last().ok_or(()), Ok);
                        if !config.allowed_invites.contains(&code.to_string()) {
                            contains = true;
                            break;
                        }
                    }
                    contains
                },
                Checks::Regex(config) => {
                    let regex = regex::Regex::new(&config.regex).map_err(|_| ())?;
                    let is_matching = regex.is_match(message_content.as_str());
                    (is_matching && config.is_matching) || (!is_matching && !config.is_matching)
                }
            };

            if !is_allowed { return Ok(()) }

        }
        println!("adsads");
        for action in automod.actions {
            match action {
                Actions::DirectMessage => {
                    let channel = discord_http.
                }
                Actions::IncreaseBucket => {}
                Actions::DeleteMessage => {}
                Actions::SendLogs => {
                    println!("1");
                    if let Some(channel) = guild_config.moderation.logs_channel {
                        println!("{channel}");
                        let avatar = get_avatar_from_member(message.author.avatar, message.author.id);
                        let embed = Embed {
                            author: Some(EmbedAuthor {
                                icon_url: Some(avatar.clone()),
                                name: format!("{}#{}", message.author.name, message.author.discriminator),
                                proxy_icon_url: Some(avatar),
                                url: None
                            }),
                            color: None,
                            description: Some(format!("Message ID: {}\nChannel: <#{}> Reason: {}", message.id, message.channel_id, automod.reason)),
                            fields: vec![],
                            footer: None,
                            image: None,
                            kind: "".to_string(),
                            provider: None,
                            thumbnail: None,
                            timestamp: None,
                            title: None,
                            url: None,
                            video: None
                        };
                        discord_http.create_message(channel).embeds(&[embed]).map_err(|_| ())?.exec().await.ok();
                    }
                }
                Actions::Timeout => {}
                Actions::Kick => {}
                Actions::Ban => {}
            };
        }

    }

    // for automod_config in &guild_config.moderation.automod {
    //
    //     match automod_config.config.clone() {
    //         AutoModeratorMethods::MessageLength(config) => {
    //
    //             let enters = message.content.lines().count();
    //             let split = message.content.len() / usize::from(config.line_len);
    //             let lines = enters + split;
    //
    //             if lines < usize::from(config.max_lines) { continue }
    //
    //             execute_action(
    //                 discord_http.clone(),
    //                 guild_config.moderation.clone(),
    //                 message.clone(),
    //                 automod_config.clone(),
    //                 "Sending too long messages"
    //             ).await?;
    //
    //             return Ok(())
    //
    //         },
    //         AutoModeratorMethods::AntiCapsLock(ref config) => {
    //
    //             if usize::from(config.min_msg_len) > message.content.len() || usize::from(config.max_msg_len) < message.content.len() {
    //                 continue
    //             }
    //
    //             let uppercase = message.content.chars().filter(|c| char::is_uppercase(c.clone())).count();
    //             let uppercase_part = uppercase * 100 / message.content.len();
    //
    //             if uppercase_part < usize::from(config.max_uppercase) { continue }
    //
    //             execute_action(
    //                 discord_http.clone(),
    //                 guild_config.moderation.clone(),
    //                 message.clone(),
    //                 automod_config.clone(),
    //                 "Turn off your CAPSLOCK"
    //             ).await?;
    //
    //         },
    //         AutoModeratorMethods::AntiInvites(ref config) => {
    //
    //             let invites = invites.find_iter(message_content.as_str());
    //
    //             let mut run = false;
    //
    //             for invite in invites {
    //                 let code = ok_or_skip!(invite.as_str().split("/").last().ok_or(()), Ok);
    //                 if !config.allowed_invites.contains(&code.to_string()) {
    //                     run = true;
    //                     break;
    //                 }
    //             }
    //
    //             if run {
    //                 execute_action(
    //                     discord_http.clone(),
    //                     guild_config.moderation.clone(),
    //                     message.clone(),
    //                     automod_config.clone(),
    //                     "Sending invites"
    //                 ).await?;
    //             }
    //
    //         },
    //         AutoModeratorMethods::AntiScamLinks => {
    //
    //             let domains = domains.find_iter(message_content.as_str());
    //             let domains = domains.map(|domain| domain.as_str().to_string()).collect();
    //
    //             if scam_domains.contains(domains).await {
    //                 execute_action(
    //                     discord_http.clone(),
    //                     guild_config.moderation.clone(),
    //                     message.clone(),
    //                     automod_config.clone(),
    //                     "Sending flagged scam links"
    //                 ).await?;
    //             }
    //
    //         }
    //     }
    // }

    Ok(())

}