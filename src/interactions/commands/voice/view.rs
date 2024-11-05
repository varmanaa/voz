use std::sync::Arc;

use eyre::Result;
use twilight_model::{
    channel::{
        message::MessageFlags,
        permission_overwrite::PermissionOverwriteType as ChannelPermissionOverwriteType,
        VideoQualityMode,
    },
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::UserMarker, Id},
};
use twilight_util::builder::{embed::EmbedBuilder, InteractionResponseDataBuilder};

use crate::{
    structs::{
        context::Context, database::ChannelPrivacy, interaction::ApplicationCommandInteraction,
    },
    utilities::time::humanize,
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
    let (privacy_permissions, privacy_text) = match voice_channel.privacy.read().clone() {
        ChannelPrivacy::Invisible => (Permissions::VIEW_CHANNEL, "Invisible"),
        ChannelPrivacy::Locked => (Permissions::CONNECT, "Locked (and visible)"),
        ChannelPrivacy::Unlocked => (Permissions::empty(), "Unlocked (and visible)"),
    };
    let permanence_text = voice_channel.permanence.read().clone().to_string();
    let mut allow_list: Vec<Id<UserMarker>> = Vec::new();
    let mut deny_list: Vec<Id<UserMarker>> = Vec::new();

    for permission_overwrite in voice_channel.permission_overwrites.read().iter() {
        if permission_overwrite
            .kind
            .ne(&ChannelPermissionOverwriteType::Member)
        {
            continue;
        }
        if permission_overwrite.id.eq(&interaction.user_id.cast()) {
            continue;
        }

        if permission_overwrite.allow.contains(privacy_permissions)
            && permission_overwrite.deny.is_empty()
        {
            allow_list.push(permission_overwrite.id.cast());
        } else if permission_overwrite.deny.contains(privacy_permissions)
            && permission_overwrite.allow.is_empty()
        {
            deny_list.push(permission_overwrite.id.cast());
        }
    }

    let allow_list_text = if allow_list.is_empty() {
        "No user has been allowed.".to_owned()
    } else if allow_list.len().eq(&1) {
        "1 user has been allowed.".to_owned()
    } else {
        format!("{} users have been allowed.", allow_list.len())
    };
    let deny_list_text = if deny_list.is_empty() {
        "No user has been denied.".to_owned()
    } else if deny_list.len().eq(&1) {
        "1 user has been denied.".to_owned()
    } else {
        format!("{} users have been denied.", deny_list.len())
    };
    let bitrate_text = format!("{}kbps", voice_channel.bitrate.read().clone() / 1_000);
    let owner_text = voice_channel
        .owner_id
        .read()
        .map_or("No owner".to_owned(), |owner_id| format!("<@{owner_id}>"));
    let slow_mode_text = match voice_channel.rate_limit_per_user.read().clone() {
        None | Some(0) => "No slow mode has been set.".to_owned(),
        Some(slow_mode) => humanize(slow_mode.into()),
    };
    let user_limit_text = match voice_channel.user_limit.read().clone() {
        None | Some(0) => "No limit has been set.".to_owned(),
        Some(1) => "1 user".to_owned(),
        Some(user_limit) => format!("{user_limit} users"),
    };
    let video_quality_mode_text = match voice_channel.video_quality_mode.read().clone() {
        VideoQualityMode::Full => "720p",
        _ => "Auto",
    };
    let voice_region_text = match voice_channel.rtc_region.read().as_deref() {
        Some("brazil") => "Brazil",
        Some("hongkong") => "Hong Kong",
        Some("india") => "India",
        Some("japan") => "Japan",
        Some("rotterdam") => "Rotterdam",
        Some("russia") => "Russia",
        Some("singapore") => "Singapore",
        Some("southafrica") => "South Africa",
        Some("sydney") => "Sydney",
        Some("us-central") => "US Central",
        Some("us-east") => "US East",
        Some("us-south") => "US South",
        Some("us-west") => "US West",
        _ => "Automatic",
    };
    let description = vec![
        format!("**Allow list:** {allow_list_text}"),
        format!("**Bitrate:** {bitrate_text}"),
        format!("**Deny list:** {deny_list_text}"),
        format!("**Owner:** {owner_text}"),
        format!("**Permanence:** {permanence_text}"),
        format!("**Privacy:** {privacy_text}"),
        format!("**Slow mode:** {slow_mode_text}"),
        format!("**User limit:** {user_limit_text}"),
        format!("**Video quality mode:** {video_quality_mode_text}"),
        format!("**Voice region:** {voice_region_text}"),
    ]
    .join("\n");
    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(description)
        .title(format!("Settings for \"{}\"", voice_channel.name.read()))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
