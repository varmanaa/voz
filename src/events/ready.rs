use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    gateway::payload::incoming::Ready,
    id::{marker::GuildMarker, Id},
};

use crate::structs::context::Context;

pub fn run(context: Arc<Context>, payload: Ready) -> Result<()> {
    let unavailable_guild_ids = payload
        .guilds
        .into_iter()
        .map(|unavailable_guild| unavailable_guild.id)
        .collect::<Vec<Id<GuildMarker>>>();

    context
        .cache
        .insert_unavailable_guilds(unavailable_guild_ids);

    println!(
        "{}#{:04} is ready!",
        payload.user.name, payload.user.discriminator
    );

    Ok(())
}
