use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use parking_lot::RwLock;
use twilight_model::{
    channel::{
        permission_overwrite::PermissionOverwrite as ChannelPermissionOverwrite, VideoQualityMode,
    },
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
};

use super::database::ChannelPrivacy;

pub struct Cache {
    pub guilds: RwLock<HashMap<Id<GuildMarker>, Arc<CachedGuild>>>,
    pub join_channels: RwLock<HashMap<Id<ChannelMarker>, Arc<CachedJoinChannel>>>,
    pub unavailable_guilds: RwLock<HashSet<Id<GuildMarker>>>,
    pub voice_channels: RwLock<HashMap<Id<ChannelMarker>, Arc<CachedVoiceChannel>>>,
    pub voice_channel_owners:
        RwLock<HashMap<(Id<GuildMarker>, Id<UserMarker>), Arc<Id<ChannelMarker>>>>,
    pub voice_states: RwLock<HashMap<(Id<GuildMarker>, Id<UserMarker>), Arc<Id<ChannelMarker>>>>,
}

pub struct CachedGuild {
    pub bot_role_id: Id<RoleMarker>,
    pub id: Id<GuildMarker>,
    pub join_channel_ids: RwLock<HashSet<Id<ChannelMarker>>>,
    pub name: RwLock<String>,
    pub voice_channel_ids: RwLock<HashSet<Id<ChannelMarker>>>,
}

#[derive(Default)]
pub struct CachedGuildUpdate {
    pub name: Option<String>,
}

pub struct CachedJoinChannel {
    pub access_role_id: RwLock<Option<Id<RoleMarker>>>,
    pub id: Id<ChannelMarker>,
    pub guild_id: Id<GuildMarker>,
    pub name: RwLock<String>,
    pub parent_id: RwLock<Option<Id<ChannelMarker>>>,
    pub permanence: RwLock<bool>,
    pub permission_overwrites: RwLock<Vec<ChannelPermissionOverwrite>>,
    pub privacy: RwLock<ChannelPrivacy>,
}

#[derive(Default)]
pub struct CachedJoinChannelUpdate {
    pub access_role_id: Option<Option<Id<RoleMarker>>>,
    pub name: Option<String>,
    pub parent_id: Option<Option<Id<ChannelMarker>>>,
    pub permanence: Option<bool>,
    pub permission_overwrites: Option<Vec<ChannelPermissionOverwrite>>,
    pub privacy: Option<ChannelPrivacy>,
}

pub struct CachedVoiceChannel {
    pub bitrate: RwLock<u32>,
    pub connected_user_ids: RwLock<HashSet<Id<UserMarker>>>,
    pub id: Id<ChannelMarker>,
    pub guild_id: Id<GuildMarker>,
    pub name: RwLock<String>,
    pub owner_id: RwLock<Option<Id<UserMarker>>>,
    pub permanence: RwLock<bool>,
    pub permission_overwrites: RwLock<Vec<ChannelPermissionOverwrite>>,
    pub privacy: RwLock<ChannelPrivacy>,
    pub rate_limit_per_user: RwLock<Option<u16>>,
    pub rtc_region: RwLock<Option<String>>,
    pub user_limit: RwLock<Option<u32>>,
    pub video_quality_mode: RwLock<VideoQualityMode>,
}

#[derive(Default)]
pub struct CachedVoiceChannelUpdate {
    pub bitrate: Option<u32>,
    pub name: Option<String>,
    pub owner_id: Option<Option<Id<UserMarker>>>,
    pub permanence: Option<bool>,
    pub permission_overwrites: Option<Vec<ChannelPermissionOverwrite>>,
    pub privacy: Option<ChannelPrivacy>,
    pub rate_limit_per_user: Option<Option<u16>>,
    pub rtc_region: Option<Option<String>>,
    pub user_limit: Option<Option<u32>>,
    pub video_quality_mode: Option<VideoQualityMode>,
}

impl Cache {
    pub fn guild(&self, id: Id<GuildMarker>) -> Option<Arc<CachedGuild>> {
        self.guilds.read().get(&id).cloned()
    }

    pub fn insert_guild(&self, bot_role_id: Id<RoleMarker>, id: Id<GuildMarker>, name: String) {
        self.guilds.write().insert(
            id,
            Arc::new(CachedGuild {
                bot_role_id,
                id,
                join_channel_ids: RwLock::new(HashSet::new()),
                name: RwLock::new(name),
                voice_channel_ids: RwLock::new(HashSet::new()),
            }),
        );
    }

    pub fn insert_join_channel(
        &self,
        access_role_id: Option<Id<RoleMarker>>,
        id: Id<ChannelMarker>,
        guild_id: Id<GuildMarker>,
        name: String,
        parent_id: Option<Id<ChannelMarker>>,
        permanence: bool,
        permission_overwrites: Vec<ChannelPermissionOverwrite>,
        privacy: ChannelPrivacy,
    ) {
        self.join_channels.write().insert(
            id,
            Arc::new(CachedJoinChannel {
                access_role_id: RwLock::new(access_role_id),
                id,
                guild_id,
                name: RwLock::new(name),
                parent_id: RwLock::new(parent_id),
                permanence: RwLock::new(permanence),
                permission_overwrites: RwLock::new(permission_overwrites),
                privacy: RwLock::new(privacy),
            }),
        );

        if let Some(guild) = self.guilds.write().get_mut(&guild_id) {
            guild.join_channel_ids.write().insert(id);
        }
    }

    pub fn insert_voice_channel(
        &self,
        bitrate: u32,
        id: Id<ChannelMarker>,
        guild_id: Id<GuildMarker>,
        name: String,
        owner_id: Option<Id<UserMarker>>,
        permanence: bool,
        permission_overwrites: Vec<ChannelPermissionOverwrite>,
        privacy: ChannelPrivacy,
        rate_limit_per_user: Option<u16>,
        rtc_region: Option<String>,
        user_limit: Option<u32>,
        video_quality_mode: VideoQualityMode,
    ) {
        self.voice_channels.write().insert(
            id,
            Arc::new(CachedVoiceChannel {
                bitrate: RwLock::new(bitrate),
                connected_user_ids: RwLock::new(HashSet::new()),
                id,
                guild_id,
                name: RwLock::new(name),
                owner_id: RwLock::new(owner_id),
                permanence: RwLock::new(permanence),
                permission_overwrites: RwLock::new(permission_overwrites),
                privacy: RwLock::new(privacy),
                rate_limit_per_user: RwLock::new(rate_limit_per_user),
                rtc_region: RwLock::new(rtc_region),
                user_limit: RwLock::new(user_limit),
                video_quality_mode: RwLock::new(video_quality_mode),
            }),
        );

        if let Some(owner_id) = owner_id {
            self.voice_channel_owners
                .write()
                .insert((guild_id, owner_id), Arc::new(id));
        }
        if let Some(guild) = self.guilds.write().get_mut(&guild_id) {
            guild.voice_channel_ids.write().insert(id);
        }
    }

    pub fn insert_voice_state(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        channel_id: Id<ChannelMarker>,
    ) {
        self.voice_states
            .write()
            .insert((guild_id, user_id), Arc::new(channel_id));

        if let Some(voice_channel) = self.voice_channels.write().get_mut(&channel_id) {
            voice_channel.connected_user_ids.write().insert(user_id);
        }
    }

    pub fn insert_unavailable_guilds(&self, ids: Vec<Id<GuildMarker>>) {
        self.unavailable_guilds.write().extend(ids);
    }

    pub fn join_channel(&self, id: Id<ChannelMarker>) -> Option<Arc<CachedJoinChannel>> {
        self.join_channels.read().get(&id).cloned()
    }

    pub fn new() -> Self {
        Self {
            guilds: RwLock::new(HashMap::new()),
            join_channels: RwLock::new(HashMap::new()),
            unavailable_guilds: RwLock::new(HashSet::new()),
            voice_channels: RwLock::new(HashMap::new()),
            voice_channel_owners: RwLock::new(HashMap::new()),
            voice_states: RwLock::new(HashMap::new()),
        }
    }

    pub fn remove_guild(&self, id: Id<GuildMarker>) {
        let Some(guild) = self.guilds.write().remove(&id) else {
            return;
        };

        for join_channel_id in guild.join_channel_ids.read().clone().into_iter() {
            self.remove_join_channel(join_channel_id);
        }

        for voice_channel_id in guild.voice_channel_ids.read().clone().into_iter() {
            self.remove_voice_channel(voice_channel_id);
        }
    }

    pub fn remove_join_channel(&self, id: Id<ChannelMarker>) {
        let Some(join_channel) = self.join_channels.write().remove(&id) else {
            return;
        };

        if let Some(guild) = self.guilds.write().get_mut(&join_channel.guild_id) {
            guild.join_channel_ids.write().remove(&id);
        }
    }

    pub fn remove_voice_channel(&self, id: Id<ChannelMarker>) {
        let Some(voice_channel) = self.voice_channels.write().remove(&id) else {
            return;
        };

        if let Some(guild) = self.guilds.write().get_mut(&voice_channel.guild_id) {
            guild.voice_channel_ids.write().remove(&id);
        }
        if let Some(owner_id) = voice_channel.owner_id.read().clone() {
            self.voice_channel_owners
                .write()
                .remove(&(voice_channel.guild_id, owner_id));
        }

        for user_id in voice_channel.connected_user_ids.read().clone().into_iter() {
            self.voice_states
                .write()
                .remove(&(voice_channel.guild_id, user_id));
        }
    }

    pub fn remove_voice_state(&self, guild_id: Id<GuildMarker>, user_id: Id<UserMarker>) {
        let Some(channel_id) = self.voice_states.write().remove(&(guild_id, user_id)) else {
            return;
        };

        if let Some(voice_channel) = self.voice_channels.write().get_mut(&channel_id) {
            voice_channel.connected_user_ids.write().remove(&user_id);
        };
    }

    pub fn update_guild(&self, id: Id<GuildMarker>, update: CachedGuildUpdate) {
        if let Some(guild) = self.guilds.write().get_mut(&id) {
            if let Some(name) = update.name {
                *guild.name.write() = name;
            }
        }
    }

    pub fn update_join_channel(&self, id: Id<ChannelMarker>, update: CachedJoinChannelUpdate) {
        if let Some(join_channel) = self.join_channels.write().get_mut(&id) {
            if let Some(access_role_id) = update.access_role_id {
                *join_channel.access_role_id.write() = access_role_id;
            }
            if let Some(name) = update.name {
                *join_channel.name.write() = name;
            }
            if let Some(parent_id) = update.parent_id {
                *join_channel.parent_id.write() = parent_id;
            }
            if let Some(permanence) = update.permanence {
                *join_channel.permanence.write() = permanence;
            }
            if let Some(permission_overwrites) = update.permission_overwrites {
                *join_channel.permission_overwrites.write() = permission_overwrites;
            }
            if let Some(privacy) = update.privacy {
                *join_channel.privacy.write() = privacy;
            }
        }
    }

    pub fn update_voice_channel(&self, id: Id<ChannelMarker>, update: CachedVoiceChannelUpdate) {
        if let Some(voice_channel) = self.voice_channels.write().get_mut(&id) {
            if let Some(bitrate) = update.bitrate {
                *voice_channel.bitrate.write() = bitrate;
            }
            if let Some(name) = update.name {
                *voice_channel.name.write() = name;
            }
            if let Some(owner_id) = update.owner_id {
                if let Some(current_owner_id) = voice_channel.owner_id.read().clone() {
                    self.voice_channel_owners
                        .write()
                        .remove(&(voice_channel.guild_id, current_owner_id));
                }

                *voice_channel.owner_id.write() = owner_id;

                if let Some(new_owner_id) = owner_id {
                    self.voice_channel_owners.write().insert(
                        (voice_channel.guild_id, new_owner_id),
                        Arc::new(voice_channel.id),
                    );
                }
            }
            if let Some(permanence) = update.permanence {
                *voice_channel.permanence.write() = permanence;
            }
            if let Some(permission_overwrites) = update.permission_overwrites {
                *voice_channel.permission_overwrites.write() = permission_overwrites;
            }
            if let Some(privacy) = update.privacy {
                *voice_channel.privacy.write() = privacy;
            }
            if let Some(rate_limit_per_user) = update.rate_limit_per_user {
                *voice_channel.rate_limit_per_user.write() = rate_limit_per_user;
            }
            if let Some(rtc_region) = update.rtc_region {
                *voice_channel.rtc_region.write() = rtc_region;
            }
            if let Some(user_limit) = update.user_limit {
                *voice_channel.user_limit.write() = user_limit;
            }
            if let Some(video_quality_mode) = update.video_quality_mode {
                *voice_channel.video_quality_mode.write() = video_quality_mode;
            }
        }
    }

    pub fn voice_channel(&self, id: Id<ChannelMarker>) -> Option<Arc<CachedVoiceChannel>> {
        self.voice_channels.read().get(&id).cloned()
    }

    pub fn voice_channel_owner(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Arc<Id<ChannelMarker>>> {
        self.voice_channel_owners
            .read()
            .get(&(guild_id, user_id))
            .cloned()
    }

    pub fn voice_state(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Arc<Id<ChannelMarker>>> {
        self.voice_states.read().get(&(guild_id, user_id)).cloned()
    }
}
