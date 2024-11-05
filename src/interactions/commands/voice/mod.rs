mod allow_member;
mod bitrate;
mod claim;
mod delete;
mod deny_member;
mod name;
mod permanence;
mod privacy;
mod remove_member;
mod slow_mode;
mod transfer;
mod user_limit;
mod video_quality_mode;
mod view;
mod voice_region;

use std::{mem::replace, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::{CommandDataOption, CommandOptionValue},
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{context::Context, interaction::ApplicationCommandInteraction};

pub async fn run(
    context: Arc<Context>,
    mut interaction: ApplicationCommandInteraction,
) -> Result<()> {
    let Some(CommandDataOption {
        name,
        value: CommandOptionValue::SubCommand(options),
    }) = interaction.data.options.clone().into_iter().nth(0)
    else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find a subcommand.")
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
    let _ = replace(&mut interaction.data.options, options);

    match name.as_str() {
        "allow-member" => allow_member::run(context, interaction).await?,
        "bitrate" => bitrate::run(context, interaction).await?,
        "claim" => claim::run(context, interaction).await?,
        "delete" => delete::run(context, interaction).await?,
        "deny-member" => deny_member::run(context, interaction).await?,
        "name" => name::run(context, interaction).await?,
        "permanence" => permanence::run(context, interaction).await?,
        "privacy" => privacy::run(context, interaction).await?,
        "remove-member" => remove_member::run(context, interaction).await?,
        "slow-mode" => slow_mode::run(context, interaction).await?,
        "transfer" => transfer::run(context, interaction).await?,
        "user-limit" => user_limit::run(context, interaction).await?,
        "video-quality-mode" => video_quality_mode::run(context, interaction).await?,
        "view" => view::run(context, interaction).await?,
        "voice-region" => voice_region::run(context, interaction).await?,
        _ => {
            let embed = EmbedBuilder::new()
                .color(0xF8F8FF)
                .description(format!(
                    "I don't have a subcommand with the name \"{name}\"."
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

            return Ok(());
        }
    }

    Ok(())
}
