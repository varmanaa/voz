use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::{
        permission_overwrite::{
            PermissionOverwrite as ChannelPermissionOverwrite,
            PermissionOverwriteType as ChannelPermissionOverwriteType,
        },
        ChannelType, VideoQualityMode,
    },
    gateway::payload::incoming::VoiceStateUpdate,
    guild::Permissions,
};

use crate::structs::{context::Context, database::ChannelPrivacy};

pub async fn run(context: Arc<Context>, payload: VoiceStateUpdate) -> Result<()> {
    let (Some(guild_id), Some(member)) = (payload.0.guild_id, payload.0.member) else {
        return Ok(());
    };

    if member.user.bot {
        return Ok(());
    }

    let Some(guild) = context.cache.guild(guild_id) else {
        return Ok(());
    };
    let user_id = member.user.id;

    if let Some(known_channel_id) = context.cache.voice_state(guild_id, user_id) {
        let known_channel_id = *known_channel_id;

        if context.cache.voice_channel(known_channel_id).is_none() {
            return Ok(());
        };

        context.cache.remove_voice_state(guild_id, user_id);

        let Some(voice_channel) = context.cache.voice_channel(known_channel_id) else {
            return Ok(());
        };

        if voice_channel.permanence.read().eq(&false)
            && voice_channel.connected_user_ids.read().is_empty()
        {
            context.client.delete_channel(known_channel_id).await?;
        }
    }
    if let Some(channel_id) = payload.0.channel_id {
        if context.cache.voice_channel(channel_id).is_some() {
            context
                .cache
                .insert_voice_state(guild_id, user_id, channel_id);
        }

        if context
            .cache
            .voice_channel_owner(guild_id, user_id)
            .is_some()
        {
            return Ok(());
        }

        let Some(join_channel) = context.cache.join_channel(channel_id) else {
            return Ok(());
        };
        let username = member.user.name;
        let name = if username.ends_with("s") {
            format!("{username}' voice")
        } else {
            format!("{username}'s voice")
        };
        let join_channel_privacy = join_channel.privacy.read().clone();
        let join_channel_parent_id = join_channel.parent_id.read().clone();
        let privacy_permissions = match join_channel_privacy {
            ChannelPrivacy::Invisible => Permissions::VIEW_CHANNEL,
            ChannelPrivacy::Locked => Permissions::CONNECT,
            ChannelPrivacy::Unlocked => Permissions::empty(),
        };
        let permission_overwrites = &[
            ChannelPermissionOverwrite {
                allow: privacy_permissions,
                deny: Permissions::empty(),
                id: guild.bot_role_id.cast(),
                kind: ChannelPermissionOverwriteType::Role,
            },
            ChannelPermissionOverwrite {
                allow: Permissions::empty(),
                deny: privacy_permissions,
                id: guild.id.cast(),
                kind: ChannelPermissionOverwriteType::Role,
            },
            ChannelPermissionOverwrite {
                allow: privacy_permissions,
                deny: Permissions::empty(),
                id: user_id.cast(),
                kind: ChannelPermissionOverwriteType::Member,
            },
        ];
        let mut voice_channel_request = context
            .client
            .create_guild_channel(guild_id, &name)
            .kind(ChannelType::GuildVoice)
            .permission_overwrites(permission_overwrites);

        if let Some(parent_id) = join_channel_parent_id {
            voice_channel_request = voice_channel_request.parent_id(parent_id);
        }

        let Ok(voice_channel_response) = voice_channel_request.await else {
            return Ok(());
        };
        let Ok(voice_channel) = voice_channel_response.model().await else {
            return Ok(());
        };
        let join_channel_permanence = join_channel.permanence.read().clone();

        context
            .database
            .insert_voice_channel(
                voice_channel.id,
                guild_id,
                Some(user_id),
                join_channel_permanence,
                join_channel_privacy.clone(),
            )
            .await?;
        context.cache.insert_voice_channel(
            voice_channel.bitrate.unwrap_or(64_000),
            voice_channel.id,
            guild_id,
            name,
            Some(user_id),
            join_channel_permanence,
            voice_channel.permission_overwrites.unwrap_or_default(),
            join_channel_privacy,
            voice_channel.rate_limit_per_user,
            voice_channel.rtc_region,
            voice_channel.user_limit,
            voice_channel
                .video_quality_mode
                .unwrap_or(VideoQualityMode::Auto),
        );
        context
            .client
            .update_guild_member(guild_id, user_id)
            .channel_id(Some(voice_channel.id))
            .await?;
    }

    Ok(())
}
