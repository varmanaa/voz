use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::GuildUpdate;

use crate::structs::{cache::CachedGuildUpdate, context::Context};

pub fn run(context: Arc<Context>, payload: GuildUpdate) -> Result<()> {
    let guild_id = payload.0.id;
    let Some(guild) = context.cache.guild(guild_id) else {
        return Ok(());
    };

    if guild.name.read().ne(&payload.0.name) {
        context.cache.update_guild(
            payload.0.id,
            CachedGuildUpdate {
                name: Some(payload.0.name),
            },
        );
    }

    Ok(())
}
