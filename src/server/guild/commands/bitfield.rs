use crate::models::config::GuildConfig;
use crate::models::config::moderation::MuteMode;

macro_rules! if_enabled {
    ($config: expr, $field: tt, $value: expr, $n: expr) => {
        if $config.enabled_commands.$field {
            $value |= (1 << $n);
        }
    };
}

const TIMEOUT_COMMAND: u32 = 1;
const MUTE_COMMAND: u32 = 2;
const WARN_COMMAND: u32 = 3;
const KICK_COMMAND: u32 = 4;
const BAN_COMMAND: u32 = 5;

const CONTEXT_MENUS: u32 = 6;

const CLEAR_COMMAND: u32 = 7;

const CASE_LAST_COMMAND: u32 = 8;
const CASE_LIST_COMMAND: u32 = 9;
const CASE_DETAILS_COMMAND: u32 = 10;
const CASE_REMOVE_COMMAND: u32 = 11;
const CASE_EDIT_COMMAND: u32 = 12;

const TOP_DAY_ME_COMMAND: u32 = 13;
const TOP_DAY_ALL_COMMAND: u32 = 14;
const TOP_WEEK_ME_COMMAND: u32 = 15;
const TOP_WEEK_ALL_COMMAND: u32 = 16;

pub fn get_enabled_bitfield(config: &GuildConfig) -> u32 {
    let mut value = 0;

    if let Some(moderation) = &config.moderation {
        match moderation.mute_mode {
            MuteMode::DependOnCommand => {
                if_enabled!(config, timeout, value, TIMEOUT_COMMAND);
                if_enabled!(config, mute, value, MUTE_COMMAND);

                if moderation.context_menu {
                    if_enabled!(config, timeout, value, CONTEXT_MENUS);
                    if_enabled!(config, mute, value, CONTEXT_MENUS);
                }
            }
            MuteMode::Timeout => {
                if_enabled!(config, timeout, value, TIMEOUT_COMMAND);
                if moderation.context_menu {
                    if_enabled!(config, mute, value, CONTEXT_MENUS);
                }
            }
            MuteMode::Role => {
                if_enabled!(config, mute, value, MUTE_COMMAND);
                if moderation.context_menu {
                    if_enabled!(config, mute, value, CONTEXT_MENUS);
                }
            }
        };

        if_enabled!(config, warn, value, WARN_COMMAND);
        if_enabled!(config, kick, value, KICK_COMMAND);
        if_enabled!(config, ban, value, BAN_COMMAND);

        if moderation.context_menu {
            if_enabled!(config, warn, value, CONTEXT_MENUS);
            if_enabled!(config, kick, value, CONTEXT_MENUS);
            if_enabled!(config, ban, value, CONTEXT_MENUS);
        }

        if_enabled!(config, clear, value, CLEAR_COMMAND);
    }

    if_enabled!(config, case_last, value, CASE_LAST_COMMAND);
    if_enabled!(config, case_list, value, CASE_LIST_COMMAND);
    if_enabled!(config, case_details, value, CASE_DETAILS_COMMAND);
    if_enabled!(config, case_remove, value, CASE_REMOVE_COMMAND);
    if_enabled!(config, case_edit, value, CASE_EDIT_COMMAND);

    if let Some(top) = &config.top {
        if top.day {
            if_enabled!(config, top_day_me, value, TOP_DAY_ME_COMMAND);
            if_enabled!(config, top_day_all, value, TOP_DAY_ALL_COMMAND);
        }

        if top.week {
            if_enabled!(config, top_week_me, value, TOP_WEEK_ME_COMMAND);
            if_enabled!(config, top_week_all, value, TOP_WEEK_ALL_COMMAND);
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use crate::server::guild::commands::bitfield::get_enabled_bitfield;
    use crate::utils::config::create_debug_config;

    #[test]
    fn test_is_equal() {
        let config1 = create_debug_config();
        let mut config2 = create_debug_config();
        assert_eq!(get_enabled_bitfield(&config1), get_enabled_bitfield(&config2));

        config2.enabled_commands.ban = false;
        assert_ne!(get_enabled_bitfield(&config1), get_enabled_bitfield(&config2));
    }
}