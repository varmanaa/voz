use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::message::MessageFlags,
    guild::Permissions,
    http::{
        interaction::{InteractionResponse, InteractionResponseType},
        permission_overwrite::{
            PermissionOverwrite as HttpPermissionOverwrite,
            PermissionOverwriteType as HttpPermissionOverwriteType,
        },
    },
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

    if context
        .cache
        .voice_channel_owner(interaction.guild.id, interaction.user_id)
        .is_some()
    {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("You already own a voice channel.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let Some(channel_id) = context
        .cache
        .voice_state(interaction.guild.id, interaction.user_id)
    else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("You are not connected to any voice channel.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let channel_id = *channel_id;
    let Some(voice_channel) = context.cache.voice_channel(channel_id) else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find this voice channel.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };

    if voice_channel.owner_id.read().is_some() {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("This voice channel already has an owner.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };

    let privacy_permissions = match voice_channel.privacy.read().clone() {
        ChannelPrivacy::Invisible => Permissions::VIEW_CHANNEL,
        ChannelPrivacy::Locked => Permissions::CONNECT,
        ChannelPrivacy::Unlocked => Permissions::empty(),
    };

    context
        .client
        .update_channel_permission(
            voice_channel.id,
            &HttpPermissionOverwrite {
                allow: Some(privacy_permissions),
                deny: None,
                id: interaction.user_id.cast(),
                kind: HttpPermissionOverwriteType::Member,
            },
        )
        .await?;

    context
        .database
        .update_voice_channel_owner_id(voice_channel.id, Some(interaction.user_id))
        .await?;
    context.cache.update_voice_channel(
        voice_channel.id,
        CachedVoiceChannelUpdate {
            owner_id: Some(Some(interaction.user_id)),
            ..Default::default()
        },
    );

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!("You are now the owner of <#{channel_id}>."))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
