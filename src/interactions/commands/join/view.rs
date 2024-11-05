use std::{str::FromStr, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue},
        interaction::application_command::CommandOptionValue,
    },
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ChannelMarker, Id},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{
    context::Context, database::ChannelPrivacy, interaction::ApplicationCommandInteraction,
};

pub async fn run(context: Arc<Context>, interaction: ApplicationCommandInteraction) -> Result<()> {
    let channel_value = match interaction
        .data
        .options
        .iter()
        .find(|&option| option.name.eq("channel"))
        .cloned()
        .map(|option| option.value)
    {
        Some(CommandOptionValue::Focused(value, _)) => {
            let lowercased_value = value.to_ascii_lowercase();
            let mut filtered_join_channels = interaction
                .guild
                .join_channel_ids
                .read()
                .clone()
                .into_iter()
                .filter_map(|channel_id| {
                    let Some(join_channel) = context.cache.join_channel(channel_id) else {
                        return None;
                    };
                    let name = join_channel.name.read().clone();

                    if !name.contains(&lowercased_value) {
                        return None;
                    }

                    Some((name, join_channel.id.to_string()))
                })
                .collect::<Vec<(String, String)>>();

            filtered_join_channels.sort();

            let choices = filtered_join_channels
                .into_iter()
                .map(|join_channel| CommandOptionChoice {
                    name: join_channel.0,
                    name_localizations: None,
                    value: CommandOptionChoiceValue::String(join_channel.1),
                })
                .collect::<Vec<CommandOptionChoice>>();
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
        Some(CommandOptionValue::String(value)) => value,
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

    let Ok(channel_id) = Id::<ChannelMarker>::from_str(&channel_value) else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I could not find a valid **channel** value.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };

    let Some(join_channel) = context.cache.join_channel(channel_id) else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I do not recognize this join channel.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let access_role_text = join_channel
        .access_role_id
        .read()
        .map_or("No access role set.".to_owned(), |access_role_id| {
            format!("<@&{access_role_id}>")
        });
    let category_text = join_channel
        .parent_id
        .read()
        .map_or("No category set.".to_owned(), |parent_id| {
            format!("<#{parent_id}>")
        });
    let permanence_text = format!("**New** voice channels from <#{channel_id}> will now have a default permanence value of **{}**.", join_channel.permanence.read());
    let privacy_text_clause = match join_channel.privacy.read().clone() {
        ChannelPrivacy::Invisible => "invisible",
        ChannelPrivacy::Locked => "locked (and visible)",
        ChannelPrivacy::Unlocked => "unlocked (and visible)",
    };
    let privacy_text = format!(
        "**New** voice channels from <#{channel_id}> will now be **{privacy_text_clause}**."
    );
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!("**Access role:** {access_role_text}\n**Category:** {category_text}\n**Permanence:** {permanence_text}\n**Privacy:** {privacy_text}"))
        .title(join_channel.name.read().clone())
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
