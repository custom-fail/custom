use serde::{Deserialize, Serialize};

/// Disabled commands indicates what commands should not be registered as guild commands
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Commands {
    pub case_details: bool,
    pub case_edit: bool,
    pub case_last: bool,
    pub case_list: bool,
    pub case_remove: bool,

    pub clear: bool,
    pub ban: bool,
    pub kick: bool,
    pub mute: bool,
    pub timeout: bool,
    pub warn: bool,

    pub top_day_all: bool,
    pub top_day_me: bool,
    pub top_week_all: bool,
    pub top_week_me: bool
}