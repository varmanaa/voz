use std::sync::Arc;

use eyre::Result;
use thousands::Separable;
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue},
        interaction::application_command::CommandOptionValue,
    },
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    structs::{context::Context, interaction::ApplicationCommandInteraction},
    utilities::constants::SLOW_MODE_OPTIONS,
};

pub async fn run(context: Arc<Context>, interaction: ApplicationCommandInteraction) -> Result<()> {
    let slow_mode = match interaction
        .data
        .options
        .iter()
        .find(|&option| option.name.eq("duration"))
        .cloned()
        .map(|option| option.value)
    {
        Some(CommandOptionValue::Focused(value, _)) => {
            let lowercased_value = value.to_ascii_lowercase();
            let mut choices: Vec<CommandOptionChoice> = Vec::with_capacity(5);

            for [name, value] in SLOW_MODE_OPTIONS.clone().into_iter() {
                if name.to_ascii_lowercase().contains(&lowercased_value) {
                    choices.push(CommandOptionChoice {
                        name,
                        name_localizations: None,
                        value: CommandOptionChoiceValue::String(value),
                    })
                }
                if choices.len().ge(&5) {
                    break;
                }
            }

            let data = InteractionResponseDataBuilder::new()
                .choices(choices)
                .build();
            let interaction_response = InteractionResponse {
                data: Some(data),
                kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
            };

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;

            return Ok(());
        }
        Some(CommandOptionValue::String(duration)) => duration.parse::<u16>()?,
        _ => return Ok(()),
    };
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
    let formatted_slow_mode = slow_mode.ne(&0u16).then_some(slow_mode);

    if voice_channel
        .rate_limit_per_user
        .read()
        .eq(&formatted_slow_mode)
    {
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
        .rate_limit_per_user(slow_mode)
        .await
        .is_err()
    {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I'm unable to set the slow mode right now. Try again in 10 minutes.")
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
            "The slow mode for <#{voice_channel_id}> is now {} second(s).",
            slow_mode.separate_with_commas(),
        ))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
