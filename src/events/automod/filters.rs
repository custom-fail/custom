use twilight_model::channel::Message;
use crate::models::config::automod::filters::{Filter, MinMax};

impl Filter {
    /// Returns `true` when value matches provided rule
    pub fn is_matching(&self, message: &Message) -> bool {
        match &self {
            Filter::MessageType(kind) => &message.kind == kind,
            Filter::MessageLength(data) => data.is_matching(message.content.len() as u16),
            Filter::Attachments(data) => data.is_matching(message.attachments.len() as u8),
            Filter::AuthorIsBot => message.author.bot,
            Filter::AuthorIsWebhook => message.webhook_id.is_some(),
            Filter::HasSticker => !message.sticker_items.is_empty(),
            Filter::Embeds(data) => data.is_matching(message.embeds.len() as u8),
            Filter::IsTTS => message.tts,
            Filter::IsInThread => message.thread.is_some(),
        }
    }
}

pub trait MinMaxConst {
    const MIN: Self;
    const MAX: Self;
}

macro_rules! impl_min_max {
    ($($name: ty),*) => {
        $(
            impl MinMaxConst for $name {
                const MIN: Self = Self::MIN;
                const MAX: Self = Self::MAX;
            }
        )*
    };
}

impl_min_max!(u8, u16);

impl<T> MinMax<T> where T: MinMaxConst, T: Copy {
    fn min(&self) -> T { self.min.unwrap_or(T::MIN).to_owned() }
    fn max(&self) -> T { self.max.unwrap_or(T::MAX).to_owned() }

    pub fn is_matching(&self, value: T) -> bool where T: PartialOrd {
        value >= self.min() && value <= self.max()
    }
}

#[cfg(test)]
mod tests {
    use crate::models::config::automod::filters::MinMax;

    #[test]
    fn test_min_max_matching() {
        let min_max = MinMax { min: Some(5u8), max: Some(10u8) };
        assert!(min_max.is_matching(5));
        assert!(min_max.is_matching(10));
        assert!(min_max.is_matching(6));
        assert!(!min_max.is_matching(4));
        assert!(!min_max.is_matching(11));

        let min_max = MinMax { min: None, max: Some(10u8) };
        assert!(min_max.is_matching(u8::MIN));
        assert!(min_max.is_matching(5));
        assert!(!min_max.is_matching(11));

        let min_max = MinMax { min: Some(5u8), max: None };
        assert!(!min_max.is_matching(4));
        assert!(min_max.is_matching(5));
        assert!(min_max.is_matching(11));
        assert!(min_max.is_matching(u8::MAX));
    }

    #[test]
    fn test_min_max_handlers() {
        let min_max = MinMax { min: Some(5u8), max: Some(10u8) };
        assert_eq!(min_max.min(), 5);
        assert_eq!(min_max.max(), 10);

        let min_max = MinMax { min: None, max: Some(10u8) };
        assert_eq!(min_max.min(), u8::MIN);
        assert_eq!(min_max.max(), 10);

        let min_max = MinMax { min: Some(5u8), max: None };
        assert_eq!(min_max.min(), 5);
        assert_eq!(min_max.max(), u8::MAX);
    }
}
