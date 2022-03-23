use twilight_model::application::component::text_input::TextInputStyle;
use twilight_model::application::component::{ActionRow, Component, TextInput};
use twilight_model::http::interaction::InteractionResponseData;

pub enum RepetitiveTextInput {
    Reason,
    Member,
    Duration
}

pub struct ModalBuilder {
    custom_id: String,
    title: String,
    inputs: Vec<TextInput>
}

impl ModalBuilder {

    pub fn new(custom_id: String, title: String) -> Self {
        Self {
            custom_id,
            title,
            inputs: vec![]
        }
    }

    pub fn add_custom_component(&self, text_input: TextInput) -> Self {
        Self {
            custom_id: self.custom_id.clone(),
            title: self.title.clone(),
            inputs: vec![self.inputs.clone(), vec![text_input]].concat()
        }
    }

    pub fn add_repetitive_component(&self, input_type: RepetitiveTextInput) -> Self {
        match input_type {
            RepetitiveTextInput::Reason => {
                self.add_custom_component(TextInput {
                    custom_id: "reason".to_string(),
                    label: "Reason".to_string(),
                    max_length: Some(512),
                    min_length: None,
                    placeholder: None,
                    required: Some(false),
                    style: TextInputStyle::Paragraph,
                    value: None
                })
            }
            RepetitiveTextInput::Duration => {
                self.add_custom_component(TextInput {
                    custom_id: "duration".to_string(),
                    label: "Duration".to_string(),
                    max_length: Some(21),
                    min_length: None,
                    placeholder: None,
                    required: Some(true),
                    style: TextInputStyle::Short,
                    value: None
                })
            },
            RepetitiveTextInput::Member => {
                self.add_custom_component(TextInput {
                    custom_id: "member".to_string(),
                    label: "Member ID".to_string(),
                    max_length: Some(21),
                    min_length: None,
                    placeholder: None,
                    required: Some(true),
                    style: TextInputStyle::Short,
                    value: None
                })
            }
        }
    }

    pub fn to_interaction_response_data(&self) -> InteractionResponseData {
        InteractionResponseData {
            allowed_mentions: None,
            attachments: None,
            choices: None,
            components: Some(
                self.inputs.iter().map(|text_input| {
                    Component::ActionRow(ActionRow {
                        components: vec![Component::TextInput(text_input.clone())]
                    })
                }).collect()
            ),
            content: None,
            custom_id: Some(self.custom_id.clone()),
            embeds: None,
            flags: None,
            title: Some(self.title.clone()),
            tts: None
        }
    }
}