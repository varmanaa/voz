use std::{str::FromStr, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue},
        interaction::application_command::CommandOptionValue,
    },
    channel::message::MessageFlags,
    guild::Permissions,
    http::{
        interaction::{InteractionResponse, InteractionResponseType},
        permission_overwrite::{
            PermissionOverwrite as HttpPermissionOverwrite,
            PermissionOverwriteType as HttpPermissionOverwriteType,
        },
    },
    id::{
        marker::{ChannelMarker, RoleMarker},
        Id,
    },
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{
    cache::CachedJoinChannelUpdate, context::Context, database::ChannelPrivacy,
    interaction::ApplicationCommandInteraction,
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
    let mut access_role_id: Option<Id<RoleMarker>> = None;

    if let Some(role_option) = interaction
        .data
        .options
        .iter()
        .find(|&option| option.name.eq("role"))
        .cloned()
    {
        if let CommandOptionValue::Role(value) = role_option.value {
            access_role_id = Some(value);
        }
    };

    let known_access_role_id = join_channel.access_role_id.read().clone();

    if known_access_role_id.eq(&access_role_id) {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("No changes have been made.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    }

    let mut embed_builder = EmbedBuilder::new().color(0xF8F8FF);

    if let Some(known_access_role_id) = known_access_role_id {
        context
            .client
            .delete_channel_permission(channel_id)
            .role(known_access_role_id)
            .await?;

        embed_builder = embed_builder.description(format!(
            "The access role for <#{channel_id}> has been removed."
        ));
    } else {
        let access_role_id = access_role_id.unwrap();
        let access_role_permissions = match join_channel.privacy.read().clone() {
            ChannelPrivacy::Invisible => Permissions::VIEW_CHANNEL,
            ChannelPrivacy::Locked => Permissions::CONNECT,
            ChannelPrivacy::Unlocked => Permissions::empty(),
        };

        context
            .client
            .update_channel_permission(
                join_channel.id,
                &HttpPermissionOverwrite {
                    allow: Some(access_role_permissions),
                    deny: None,
                    id: access_role_id.cast(),
                    kind: HttpPermissionOverwriteType::Role,
                },
            )
            .await?;

        embed_builder = embed_builder.description(format!(
            "The access role for <#{channel_id}> is now <@&{access_role_id}>."
        ));
    }

    context
        .database
        .update_join_channel_access_role_id(channel_id, access_role_id)
        .await?;
    context.cache.update_join_channel(
        channel_id,
        CachedJoinChannelUpdate {
            access_role_id: Some(access_role_id),
            ..Default::default()
        },
    );
    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed_builder.build()]))
        .await?;

    Ok(())
}
