use std::{str::FromStr, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::{
        command::{CommandOptionChoice, CommandOptionChoiceValue},
        interaction::application_command::CommandOptionValue,
    },
    channel::{
        message::MessageFlags,
        permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    },
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ChannelMarker, Id},
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
    let mut privacy = ChannelPrivacy::Unlocked;

    if let Some(level_option) = interaction
        .data
        .options
        .iter()
        .find(|&option| option.name.eq("level"))
        .cloned()
    {
        if let CommandOptionValue::String(value) = level_option.value {
            if value.eq("invisible") {
                privacy = ChannelPrivacy::Invisible
            } else if value.eq("locked") {
                privacy = ChannelPrivacy::Locked
            }
        }
    };
    if join_channel.privacy.read().eq(&privacy) {
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

    let mut permission_overwrites = join_channel.permission_overwrites.read().clone();
    let (common_permissions, privacy_text) = match privacy {
        ChannelPrivacy::Invisible => (Permissions::VIEW_CHANNEL, "invisible"),
        ChannelPrivacy::Locked => (Permissions::CONNECT, "locked (and visible)"),
        ChannelPrivacy::Unlocked => (Permissions::empty(), "unlocked (and visible)"),
    };

    for permission_overwrite in permission_overwrites.iter_mut() {
        if permission_overwrite
            .kind
            .ne(&ChannelPermissionOverwriteType::Role)
        {
            continue;
        }
        if permission_overwrite
            .id
            .eq(&interaction.guild.bot_role_id.cast())
        {
            permission_overwrite.allow = common_permissions;
        }
        if permission_overwrite.id.eq(&interaction.guild.id.cast()) {
            permission_overwrite.deny = common_permissions;
        }
        if join_channel
            .access_role_id
            .read()
            .is_some_and(|access_role_id| permission_overwrite.id.eq(&access_role_id.cast()))
        {
            permission_overwrite.allow = common_permissions;
        }
    }

    if context
        .client
        .update_channel(channel_id)
        .permission_overwrites(&permission_overwrites)
        .await
        .is_err()
    {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I am unable to set the privacy of this join channel.")
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
        .update_join_channel_privacy(channel_id, privacy.clone())
        .await?;
    context.cache.update_join_channel(
        channel_id,
        CachedJoinChannelUpdate {
            privacy: Some(privacy),
            ..Default::default()
        },
    );

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!(
            "**New** voice channels from <#{channel_id}> will now be **{privacy_text}**."
        ))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
