use std::collections::HashMap;

use twilight_model::id::Id;

use crate::models::config::{GuildConfig, moderation::{Moderation, MuteMode}, automod::{AutoModeration, AutoModerationRule, ignore::{Ignore, IgnoreMode}, actions::{ActionMetadata, Action, IncreaseBucket, IncreaseBucketAmount, BucketAction, Timeout}}, activity::{Levels, Top}};
use crate::models::config::commands::Commands;

#[allow(dead_code)]

/// Creates config for warns
pub fn create_debug_config() -> GuildConfig {
    GuildConfig {
        guild_id: Id::new(759848600833490945),
        application_id: None,
        enabled_commands: Commands {
            case_details: true,
            case_edit: true,
            case_last: true,
            case_list: true,
            case_remove: true,
            clear: true,
            ban: true,
            kick: true,
            mute: true,
            timeout: true,
            warn: true,
            top_day_all: true,
            top_day_me: true,
            top_week_all: true,
            top_week_me: true,
        },
        moderation: Some(Moderation {
            automod: Some(AutoModeration {
                rules: vec![AutoModerationRule {
                    basic_type: None,
                    check_on_edit: true,
                    filters: vec![],
                    checks: vec![],
                    actions: vec![
                        ActionMetadata {
                            action: Action::IncreaseBucket(IncreaseBucket {
                                key: "mentions".to_owned(),
                                amount: IncreaseBucketAmount::Mentions,
                                per_channel: false,
                                duration: 5
                            }),
                            sync: false
                        }
                    ],
                    ignore: Some(Ignore {
                        channels: vec![Id::new(981950096801406979)],
                        channels_ignore_mode: IgnoreMode::BlackList,
                        roles: vec![Id::new(981950094888820797)],
                        users: vec![]
                    }),
                    reason: "test".to_string(),
                    name: "test".to_string()
                }],
                bucket_actions: HashMap::from([
                    ("mentions".to_owned(), BucketAction {
                        actions: vec![ActionMetadata {
                            action: Action::Timeout(Timeout {
                                duration: 5000
                            }),
                            sync: false
                        }, ActionMetadata {
                            action: Action::DeleteMessage,
                            sync: false
                        }],
                        reason: "Too many mentions".to_string(),
                        limit: 5,
                    })
                ]),
                logs_channel: Some(Id::new(981950096801406979)),
                ignore: None,
            }),
            mute_mode: MuteMode::DependOnCommand,
            mute_role: None,
            native_support: true,
            logs_channel: Some(Id::new(981950096801406979)),
            dm_case: true,
            context_menu: true,
        }),
        premium: true,
        levels: Some(Levels {
            xp_timeout: 30,
            xp_min: 5,
            xp_max: 5,
        }),
        top: Some(Top {
            week: true,
            day: true,
            webhook_url: String::new(),
        }),
    }
}