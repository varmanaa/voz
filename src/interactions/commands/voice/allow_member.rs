use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::{
        message::MessageFlags,
        permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
    },
    guild::Permissions,
    http::{
        interaction::{InteractionResponse, InteractionResponseType},
        permission_overwrite::{
            PermissionOverwrite as HttpPermissionOverwrite,
            PermissionOverwriteType as HttpPermissionOverwriteType,
        },
    },
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::structs::{
    context::Context, database::ChannelPrivacy, interaction::ApplicationCommandInteraction,
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
                "<@{member_id}> may not be allowed permission in this voice channel."
            ))
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    }

    let (mut member_allow, mut member_deny) = voice_channel
        .permission_overwrites
        .read()
        .clone()
        .into_iter()
        .find(|permission_overwrite| {
            permission_overwrite
                .kind
                .eq(&ChannelPermissionOverwriteType::Member)
                && permission_overwrite.id.eq(&member_id.cast())
        })
        .map_or(
            (Permissions::empty(), Permissions::empty()),
            |permission_overwrite| (permission_overwrite.allow, permission_overwrite.deny),
        );
    let permissions = match voice_channel.privacy.read().clone() {
        ChannelPrivacy::Invisible => Permissions::VIEW_CHANNEL,
        ChannelPrivacy::Locked => Permissions::CONNECT,
        ChannelPrivacy::Unlocked => Permissions::empty(),
    };

    if member_allow.contains(permissions) {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description(format!(
                "<@{member_id}> is already allowed permission in this voice channel."
            ))
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    }

    member_allow.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);
    member_deny.remove(Permissions::CONNECT | Permissions::VIEW_CHANNEL);
    member_allow = member_allow.union(permissions);

    context
        .client
        .update_channel_permission(
            voice_channel.id,
            &HttpPermissionOverwrite {
                allow: Some(member_allow),
                deny: Some(member_deny),
                id: member_id.cast(),
                kind: HttpPermissionOverwriteType::Member,
            },
        )
        .await?;

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!(
            "<@{member_id}> is now allowed permission in this voice channel."
        ))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
