use std::{collections::HashSet, sync::Arc};

use eyre::Result;
use twilight_model::{
    application::interaction::application_command::CommandOptionValue,
    channel::{
        message::MessageFlags,
        permission_overwrite::{
            PermissionOverwrite as ChannelPermissionOverwrite,
            PermissionOverwriteType as ChannelPermissionOverwriteType,
        },
        ChannelType,
    },
    guild::Permissions,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{
        marker::{ChannelMarker, GenericMarker, RoleMarker},
        Id,
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

    let join_channel_count = interaction.guild.join_channel_ids.read().len();

    if join_channel_count.ge(&3) {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description(
                "This server has a limit of **three** join channels and is already at the limit.",
            )
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    }

    let mut name = format!("join-{}", join_channel_count + 1);
    let mut access_role_id: Option<Id<RoleMarker>> = None;
    let mut parent_id: Option<Id<ChannelMarker>> = None;
    let mut permanence = false;
    let mut privacy = ChannelPrivacy::Unlocked;
    let bot_and_everyone_role_ids: HashSet<Id<GenericMarker>> = HashSet::from_iter(vec![
        interaction.guild.bot_role_id.cast(),
        interaction.guild.id.cast(),
    ]);

    for option in interaction.data.options.into_iter() {
        match (option.name.as_str(), option.value) {
            ("name", CommandOptionValue::String(value)) => {
                name = value;
            }
            ("access-role", CommandOptionValue::Role(value)) => {
                if bot_and_everyone_role_ids.contains(&value.cast()) {
                    let embed = EmbedBuilder::new()
                        .color(0xF8F8FF)
                        .description("This role may not be used as an access role.")
                        .build();

                    context
                        .interaction_client()
                        .update_response(&interaction.token)
                        .embeds(Some(&[embed]))
                        .await?;

                    return Ok(());
                }

                access_role_id = Some(value);
            }
            ("category", CommandOptionValue::Channel(value)) => {
                parent_id = Some(value);
            }
            ("permanence", CommandOptionValue::Boolean(value)) => {
                permanence = value;
            }
            ("privacy", CommandOptionValue::String(value)) => {
                if value.eq("invisible") {
                    privacy = ChannelPrivacy::Invisible
                } else if value.eq("locked") {
                    privacy = ChannelPrivacy::Invisible
                }
            }
            _ => continue,
        }
    }

    let common_permissions = match privacy {
        ChannelPrivacy::Invisible => Permissions::VIEW_CHANNEL,
        ChannelPrivacy::Locked => Permissions::CONNECT,
        ChannelPrivacy::Unlocked => Permissions::empty(),
    };
    let guild_id = interaction.guild.id;
    let mut permission_overwrites = vec![
        ChannelPermissionOverwrite {
            allow: common_permissions,
            deny: Permissions::empty(),
            id: interaction.guild.bot_role_id.cast(),
            kind: ChannelPermissionOverwriteType::Role,
        },
        ChannelPermissionOverwrite {
            allow: Permissions::empty(),
            deny: common_permissions,
            id: guild_id.cast(),
            kind: ChannelPermissionOverwriteType::Role,
        },
    ];

    if let Some(access_role_id) = access_role_id {
        permission_overwrites.push(ChannelPermissionOverwrite {
            allow: common_permissions,
            deny: Permissions::empty(),
            id: access_role_id.cast(),
            kind: ChannelPermissionOverwriteType::Role,
        });
    }

    let request = context
        .client
        .create_guild_channel(guild_id, &name)
        .kind(ChannelType::GuildVoice)
        .permission_overwrites(&permission_overwrites);
    let Ok(response) = request.await else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I am unable to create the request.")
            .build();

        context
            .interaction_client()
            .update_response(&interaction.token)
            .embeds(Some(&[embed]))
            .await?;

        return Ok(());
    };
    let Ok(join_channel) = response.model().await else {
        let embed = EmbedBuilder::new()
            .color(0xF8F8FF)
            .description("I am unable to create the join channel.")
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
        .insert_join_channel(
            join_channel.id,
            guild_id,
            access_role_id,
            parent_id,
            permanence,
            privacy.clone(),
        )
        .await?;
    context.cache.insert_join_channel(
        access_role_id,
        join_channel.id,
        guild_id,
        name,
        parent_id,
        permanence,
        join_channel.permission_overwrites.unwrap_or_default(),
        privacy,
    );

    let embed = EmbedBuilder::new()
        .color(0xF8F8FF)
        .description(format!("I have created <#{}>.", join_channel.id))
        .build();

    context
        .interaction_client()
        .update_response(&interaction.token)
        .embeds(Some(&[embed]))
        .await?;

    Ok(())
}
