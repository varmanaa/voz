use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{context::Context, interaction::ApplicationCommandInteraction};

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

    let Some(CommandOptionValue::Integer(limit)) = interaction
        .data
        .options
        .into_iter()
        .find(|option| option.name.eq("limit"))
        .map(|option| option.value)
    else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find a valid **limit** value.")
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
    let user_limit = limit as u32;
    let formatted_user_limit = user_limit.ne(&0u32).then_some(user_limit);

    if voice_channel.user_limit.read().eq(&formatted_user_limit) {
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

    if context
        .client
        .update_channel(voice_channel.id)
        .user_limit(user_limit as u16)
        .await
        .is_err()
    {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I'm unable to set the user limit right now. Try again in 10 minutes.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!(
            "The user limit for <#{voice_channel_id}> is now {user_limit} user(s).",
        ))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
