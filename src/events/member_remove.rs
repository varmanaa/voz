use std::sync::Arc;

use eyre::Result;
use twilight_model::{gateway::payload::incoming::MemberRemove, user::User};

use crate::structs::{cache::CachedVoiceChannelUpdate, context::Context};

pub async fn run(context: Arc<Context>, payload: MemberRemove) -> Result<()> {
    let MemberRemove {
        guild_id,
        user: User { id: user_id, .. },
    } = payload;
    let Some(channel_id) = context.cache.voice_channel_owner(guild_id, user_id) else {
        return Ok(());
    };
    let channel_id = *channel_id;

    context
        .database
        .update_voice_channel_owner_id(channel_id, None)
        .await?;
    context.cache.update_voice_channel(
        channel_id,
        CachedVoiceChannelUpdate {
            owner_id: Some(None),
            ..Default::default()
        },
    );

    Ok(())
}
