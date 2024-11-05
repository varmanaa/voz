use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::ChannelDelete;

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: ChannelDelete) -> Result<()> {
    let channel_id = payload.0.id;

    if context.cache.join_channel(channel_id).is_some() {
        context.database.remove_join_channel(channel_id).await?;
        context.cache.remove_join_channel(channel_id);
    } else if context.cache.voice_channel(channel_id).is_some() {
        context.database.remove_voice_channel(channel_id).await?;
        context.cache.remove_voice_channel(channel_id);
    }

    Ok(())
}
