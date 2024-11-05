use std::sync::Arc;

use eyre::Result;
use twilight_model::gateway::payload::incoming::RoleDelete;

use crate::structs::{cache::CachedJoinChannelUpdate, context::Context};

pub async fn run(context: Arc<Context>, payload: RoleDelete) -> Result<()> {
    let RoleDelete { guild_id, role_id } = payload;
    let Some(guild) = context.cache.guild(guild_id) else {
        return Ok(());
    };
    let guild_join_channel_ids = guild.join_channel_ids.read().clone();

    for channel_id in guild_join_channel_ids {
        let Some(join_channel) = context.cache.join_channel(channel_id) else {
            continue;
        };
        let Some(access_role_id) = join_channel.access_role_id.read().clone() else {
            continue;
        };

        if access_role_id.eq(&role_id) {
            context
                .database
                .update_join_channel_access_role_id(channel_id, None)
                .await?;
            context.cache.update_join_channel(
                channel_id,
                CachedJoinChannelUpdate {
                    access_role_id: Some(None),
                    ..Default::default()
                },
            );
        }
    }

    Ok(())
}
