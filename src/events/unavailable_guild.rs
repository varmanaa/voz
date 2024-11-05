use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::UnavailableGuild;

use crate::structs::context::Context;

pub fn run(context: Arc<Context>, payload: UnavailableGuild) -> Result<()> {
    context.cache.insert_unavailable_guilds(vec![payload.id]);

    Ok(())
}
