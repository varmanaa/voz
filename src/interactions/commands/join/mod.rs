mod access_role;
mod category;
mod create;
mod name;
mod permanence;
mod privacy;
mod remove;
mod view;

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
        "access-role" => access_role::run(context, interaction).await?,
        "category" => category::run(context, interaction).await?,
        "create" => create::run(context, interaction).await?,
        "name" => name::run(context, interaction).await?,
        "permanence" => permanence::run(context, interaction).await?,
        "privacy" => privacy::run(context, interaction).await?,
        "remove" => remove::run(context, interaction).await?,
        "view" => view::run(context, interaction).await?,
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
