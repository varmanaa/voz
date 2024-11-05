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
                "<@{member_id}> may not be removed from this voice channel."
            ))
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    }

    let mut has_changed = false;
    let does_user_permission_exist = voice_channel
        .permission_overwrites
        .read()
        .clone()
        .into_iter()
        .any(|permission_overwrite| {
            permission_overwrite
                .kind
                .eq(&ChannelPermissionOverwriteType::Member)
                && permission_overwrite.id.eq(&member_id.cast())
        });

    if does_user_permission_exist {
        context
            .client
            .delete_channel_permission(voice_channel.id)
            .member(member_id)
            .await?;

        has_changed = true;
    }
    if let Some(channel_id) = context.cache.voice_state(interaction.guild.id, member_id) {
        if voice_channel.id.eq(&*channel_id) {
            context
                .client
                .update_guild_member(interaction.guild.id, member_id)
                .channel_id(None)
                .await?;

            has_changed = true;
        }
    }

    let description = if has_changed {
        format!("<@{member_id}> has been removed of this voice channel.")
    } else {
        "No change has been made.".to_owned()
    };
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description)
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
