use std::{collections::HashMap, sync::Arc};

use eyre::Result;
use twilight_model::{
    channel::{ChannelType, VideoQualityMode},
    gateway::payload::incoming::GuildCreate,
    id::{marker::ChannelMarker, Id},
};

use crate::structs::context::Context;

pub async fn run(context: Arc<Context>, payload: GuildCreate) -> Result<()> {
    let guild_id = payload.0.id;
    let Some(bot_role_id) = payload.0.roles.into_iter().find_map(|role| {
        if !role.managed {
            return None;
        }

        let Some(tags) = &role.tags else {
            return None;
        };

        tags.bot_id
            .is_some_and(|bot_id| bot_id.eq(&context.application_id.cast()))
            .then(|| role.id)
    }) else {
        context.client.leave_guild(guild_id).await?;

        return Ok(());
    };

    context
        .cache
        .insert_guild(bot_role_id, guild_id, payload.0.name);

    let filtered_guild_channels =
        payload
            .0
            .channels
            .into_iter()
            .fold(HashMap::new(), |mut acc, channel| {
                if channel.kind.eq(&ChannelType::GuildVoice) {
                    acc.insert(
                        channel.id,
                        (
                            channel.bitrate.unwrap_or(64_000),
                            channel.name.unwrap_or_default(),
                            channel.permission_overwrites.unwrap_or_default(),
                            channel.rate_limit_per_user,
                            channel.rtc_region,
                            channel.user_limit,
                            channel.video_quality_mode.unwrap_or(VideoQualityMode::Auto),
                        ),
                    );
                };

                acc
            });
    let filtered_guild_channel_ids = filtered_guild_channels
        .keys()
        .map(|id| id.to_owned())
        .collect::<Vec<Id<ChannelMarker>>>();

    context
        .database
        .remove_unknown_channels(filtered_guild_channel_ids, guild_id)
        .await?;

    for join_channel in context.database.guild_join_channels(guild_id).await? {
        let (name, permission_overwrites) =
            if let Some(filtered_guild_channel) = filtered_guild_channels.get(&join_channel.id) {
                (
                    filtered_guild_channel.1.to_owned(),
                    filtered_guild_channel.2.to_owned(),
                )
            } else {
                (String::new(), Vec::new())
            };

        context.cache.insert_join_channel(
            join_channel.access_role_id,
            join_channel.id,
            join_channel.guild_id,
            name,
            join_channel.parent_id,
            join_channel.permanence,
            permission_overwrites,
            join_channel.privacy,
        );
    }
    for voice_channel in context.database.guild_voice_channels(guild_id).await? {
        let (
            bitrate,
            name,
            permission_overwrites,
            rate_limit_per_user,
            rtc_region,
            user_limit,
            video_quality_mode,
        ) = if let Some(filtered_guild_channel) = filtered_guild_channels.get(&voice_channel.id) {
            filtered_guild_channel.to_owned()
        } else {
            (
                64_000,
                String::new(),
                Vec::new(),
                None,
                None,
                None,
                VideoQualityMode::Auto,
            )
        };

        context.cache.insert_voice_channel(
            bitrate,
            voice_channel.id,
            voice_channel.guild_id,
            name,
            voice_channel.owner_id,
            voice_channel.permanence,
            permission_overwrites,
            voice_channel.privacy,
            rate_limit_per_user,
            rtc_region,
            user_limit,
            video_quality_mode,
        );
    }
    for voice_state in payload.0.voice_states {
        let Some(channel_id) = voice_state.channel_id else {
            continue;
        };

        if context.cache.voice_channel(channel_id).is_some() {
            context
                .cache
                .insert_voice_state(guild_id, voice_state.user_id, channel_id);
        }
    }

    Ok(())
}
