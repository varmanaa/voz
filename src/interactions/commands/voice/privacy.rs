use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::{
        message::MessageFlags,
        permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    },
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{
    cache::CachedVoiceChannelUpdate, context::Context, database::ChannelPrivacy,
    interaction::ApplicationCommandInteraction,
};

pub async fn run(context: Arc<Context>, interaction: ApplicationCommandInteraction) -> Result<()> {
    let interaction_response_data = InteractionResponseDataBuilder::new()
        .flags(MessageFlags::EPHEMERAL)
        .build();
    let interaction_response = InteractionResponse {
        data: Some(interaction_response_data),
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
    };

    context
        .interaction_client()
        .create_response(interaction.id, &interaction.token, &interaction_response)
        .await?;

    let Some(CommandOptionValue::String(level)) = interaction
        .data
        .options
        .into_iter()
        .find(|option| option.name.eq("level"))
        .map(|option| option.value)
    else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find a valid **level** value.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let Some(voice_channel_id) = context
        .cache
        .voice_channel_owner(interaction.guild.id, interaction.user_id)
    else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("You do not own a voice channel.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let Some(voice_channel) = context.cache.voice_channel(*voice_channel_id) else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find your voice channel.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let (common_permissions, formatted_privacy, privacy_text) = match level.as_str() {
        "invisible" => (
            Permissions::VIEW_CHANNEL,
            ChannelPrivacy::Invisible,
            "invisible",
        ),
        "locked" => (
            Permissions::CONNECT,
            ChannelPrivacy::Locked,
            "locked (and visible)",
        ),
        _ => (
            Permissions::empty(),
            ChannelPrivacy::Unlocked,
            "unlocked (and visible)",
        ),
    };

    if voice_channel.privacy.read().eq(&formatted_privacy) {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("No change has been made.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    }

    let mut permission_overwrites = voice_channel.permission_overwrites.read().clone();

    for permission_overwrite in permission_overwrites.iter_mut() {
        if permission_overwrite
            .kind
            .eq(&ChannelPermissionOverwriteType::Member)
        {
            if voice_channel
                .owner_id
                .read()
                .is_some_and(|owner_id| permission_overwrite.id.eq(&owner_id.cast()))
            {
                permission_overwrite.allow = common_permissions;
            }
            if permission_overwrite.deny.is_empty() && permission_overwrite.allow.is_empty() {
                continue;
            }
            if permission_overwrite
                .deny
                .contains(Permissions::VIEW_CHANNEL)
            {
                continue;
            }

            permission_overwrite.allow = common_permissions;
        } else {
            if permission_overwrite
                .id
                .eq(&interaction.guild.bot_role_id.cast())
            {
                permission_overwrite.allow = common_permissions;
            }
            if permission_overwrite.id.eq(&interaction.guild.id.cast()) {
                permission_overwrite.deny = common_permissions;
            }
        }
    }

    if context
        .client
        .update_channel(voice_channel.id)
        .permission_overwrites(&permission_overwrites)
        .await
        .is_err()
    {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I'm unable to set the privacy level right now. Try again in 10 minutes.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };

    context
        .database
        .update_voice_channel_privacy(voice_channel.id, formatted_privacy.clone())
        .await?;
    context.cache.update_voice_channel(
        voice_channel.id,
        CachedVoiceChannelUpdate {
            privacy: Some(formatted_privacy),
            ..Default::default()
        },
    );

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!("<#{voice_channel_id}> is now **{privacy_text}**."))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
