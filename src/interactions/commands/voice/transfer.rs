use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::{
        message::MessageFlags,
        permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    },
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{
    cache::CachedVoiceChannelUpdate, context::Context, interaction::ApplicationCommandInteraction,
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

    let Some(CommandOptionValue::User(member_id)) = interaction
        .data
        .options
        .into_iter()
        .find(|option| option.name.eq("member"))
        .map(|option| option.value)
    else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find a valid **member** value.")
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

    if context.application_id.eq(&member_id.cast())
        || voice_channel
            .owner_id
            .read()
            .is_some_and(|owner_id| owner_id.eq(&member_id))
    {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description(format!(
                "You may not transfer this voice channel to <@{member_id}>."
            ))
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
        if permission_overwrite.id.eq(&interaction.user_id.cast())
            && permission_overwrite
                .kind
                .eq(&ChannelPermissionOverwriteType::Member)
        {
            permission_overwrite.id = member_id.cast();
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
            .description(
                "I'm unable to transfer your voice channel right now. Try again in 10 minutes.",
            )
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
        .update_voice_channel_owner_id(voice_channel.id, Some(member_id))
        .await?;
    context.cache.update_voice_channel(
        voice_channel.id,
        CachedVoiceChannelUpdate {
            owner_id: Some(Some(member_id)),
            ..Default::default()
        },
    );

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!(
            "<@{member_id}> is now the owner of <#{voice_channel_id}>.",
        ))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
