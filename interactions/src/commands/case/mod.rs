use twilight_model::application::interaction::application_command::InteractionMember;
use twilight_model::application::interaction::ApplicationCommand;
use twilight_model::id::Id;
use twilight_model::id::marker::UserMarker;

pub mod last;
pub mod details;

pub fn get_member_from_command_data(interaction: Box<ApplicationCommand>) -> Result<(Id<UserMarker>, InteractionMember), String> {
    Ok(Vec::from_iter(interaction.data.resolved.ok_or("No member specified".to_string())?
        .members.into_iter()).first().cloned().ok_or("Cannot find member information")?)
}