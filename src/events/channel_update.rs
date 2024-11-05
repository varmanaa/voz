use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::ChannelUpdate;

use crate::structs::{
    cache::{CachedJoinChannelUpdate, CachedVoiceChannelUpdate},
    context::Context,
};

pub fn run(context: Arc<Context>, payload: ChannelUpdate) -> Result<()> {
    let channel_id = payload.0.id;
    let name = payload.0.name;
    let permission_overwrites = payload.0.permission_overwrites.unwrap_or_default();

    if context.cache.join_channel(channel_id).is_some() {
        context.cache.update_join_channel(
            channel_id,
            CachedJoinChannelUpdate {
                name,
                permission_overwrites: Some(permission_overwrites),
                ..Default::default()
            },
        );
    } else if context.cache.voice_channel(channel_id).is_some() {
        context.cache.update_voice_channel(
            channel_id,
            CachedVoiceChannelUpdate {
                bitrate: payload.0.bitrate,
                name,
                permission_overwrites: Some(permission_overwrites),
                rate_limit_per_user: Some(payload.0.rate_limit_per_user),
                rtc_region: Some(payload.0.rtc_region),
                user_limit: Some(payload.0.user_limit),
                video_quality_mode: payload.0.video_quality_mode,
                ..Default::default()
            },
        );
    }

    Ok(())
}
