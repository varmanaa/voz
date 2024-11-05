use std::sync::Arc;

use twilight_model::{
    application::interaction::application_command::CommandData,
    channel::Channel,
    id::{
        marker::{InteractionMarker, UserMarker},
        Id,
    },
};

use super::cache::CachedGuild;

pub struct ApplicationCommandInteraction {
    pub channel: Channel,
    pub data: Box<CommandData>,
    pub guild: Arc<CachedGuild>,
    pub id: Id<InteractionMarker>,
    pub token: String,
    pub user_id: Id<UserMarker>,
}
