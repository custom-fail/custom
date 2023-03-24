use twilight_model::{gateway::payload::incoming::MessageUpdate, channel::{Message, message::MessageType}};

pub trait ConvertToMessage {
    fn convert(self) -> Result<Message, ()>;
}

impl ConvertToMessage for Box<MessageUpdate> {
    fn convert(self) -> Result<Message, ()> {
        let author = self.author.ok_or(())?;
        let timestamp = self.timestamp.ok_or(())?;
        Ok(Message {
            activity: None,
            application: None,
            application_id: None,
            attachments: self.attachments.unwrap_or_default(),
            author,
            channel_id: self.channel_id,
            components: vec![],
            content: self.content.unwrap_or_default(),
            edited_timestamp: self.edited_timestamp,
            embeds: self.embeds.unwrap_or_default(),
            flags: None,
            guild_id: self.guild_id,
            id: self.id,
            interaction: None,
            kind: self.kind.unwrap_or(MessageType::Regular),
            member: None,
            mention_channels: vec![],
            mention_everyone: self.mention_everyone.unwrap_or(false),
            mention_roles: self.mention_roles.unwrap_or_default(),
            mentions: self.mentions.unwrap_or_default(),
            pinned: self.pinned.unwrap_or(false),
            reactions: vec![],
            reference: None,
            referenced_message: None,
            role_subscription_data: None,
            sticker_items: vec![],
            timestamp,
            thread: None,
            tts: false,
            webhook_id: None,
        })
    }
}