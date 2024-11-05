use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::InteractionData,
    channel::message::MessageFlags,
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    interactions::commands::*,
    structs::{context::Context, interaction::ApplicationCommandInteraction},
};

pub async fn run(context: Arc<Context>, payload: InteractionCreate) -> Result<()> {
    let interaction = payload.0;
    let (Some(channel), Some(guild), Some(user_id)) = (
        interaction.channel,
        interaction
            .guild_id
            .and_then(|guild_id| context.cache.guild(guild_id)),
        interaction
            .member
            .and_then(|member| member.user.and_then(|user| Some(user.id))),
    ) else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("Please kick and re-invite me.")
            .build();
        let interaction_response_data = InteractionResponseDataBuilder::new()
            .embeds(vec![embed])
            .flags(MessageFlags::EPHEMERAL)
            .build();
        let interaction_response = InteractionResponse {
            data: Some(interaction_response_data),
            kind: InteractionResponseType::ChannelMessageWithSource,
        };

        context
            .interaction_client()
            .create_response(interaction.id, &interaction.token, &interaction_response)
            .await?;

        return Ok(());
    };

    match interaction.data {
        Some(InteractionData::ApplicationCommand(data)) => {
            let interaction = ApplicationCommandInteraction {
                channel,
                data,
                guild,
                id: interaction.id,
                token: interaction.token,
                user_id,
            };

            handle_application_command(context, interaction).await?;
        }
        _ => {
            let embed = EmbedBuilder::new()
                .color(0xF8F8FF)
                .description("I don't recognize this interaction.")
                .build();
            let interaction_response_data = InteractionResponseDataBuilder::new()
                .embeds(vec![embed])
                .flags(MessageFlags::EPHEMERAL)
                .build();
            let interaction_response = InteractionResponse {
                data: Some(interaction_response_data),
                kind: InteractionResponseType::ChannelMessageWithSource,
            };

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}

async fn handle_application_command(
    context: Arc<Context>,
    interaction: ApplicationCommandInteraction,
) -> Result<()> {
    let command_name = interaction.data.name.as_str();

    match command_name {
        "join" => join::run(context, interaction).await?,
        "voice" => voice::run(context, interaction).await?,
        _ => {
            let embed = EmbedBuilder::new()
                .color(0xF8F8FF)
                .description(format!(
                    "I don't have a command with the name \"{command_name}\"."
                ))
                .build();
            let interaction_response_data = InteractionResponseDataBuilder::new()
                .embeds(vec![embed])
                .flags(MessageFlags::EPHEMERAL)
                .build();
            let interaction_response = InteractionResponse {
                data: Some(interaction_response_data),
                kind: InteractionResponseType::ChannelMessageWithSource,
            };

            context
                .interaction_client()
                .create_response(interaction.id, &interaction.token, &interaction_response)
                .await?;
        }
    }

    Ok(())
}
