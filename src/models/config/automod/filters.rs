use serde::{Serialize, Deserialize};
use twilight_model::channel::message::MessageType;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FilterMetadata {
    #[serde(flatten)]
    pub filter: Filter,
    pub negate: bool
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(tag = "type")]
pub enum Filter {
    MessageType(MessageType),
    MessageLength(U16MinMax),
    Attachments(U8MinMax),
    AuthorIsBot,
    AuthorIsWebhook,
    HasSticker,
    Embeds(U8MinMax),
    IsTTS,
    IsInThread
}

type U8MinMax = MinMax<u8>;
type U16MinMax = MinMax<u16>;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MinMax<T> {
    pub min: Option<T>,
    pub max: Option<T>
}
