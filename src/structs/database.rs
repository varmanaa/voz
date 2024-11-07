use std::str::FromStr;

use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use eyre::{Result, WrapErr};
use tokio_postgres::{
    types::{FromSql, ToSql},
    Config, NoTls, Row,
};
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
    Id,
};

use crate::utilities::constants::DATABASE_URL;

#[derive(Clone, Debug, Eq, FromSql, PartialEq, ToSql)]
#[postgres(name = "channel_privacy")]
pub enum ChannelPrivacy {
    #[postgres(name = "invisible")]
    Invisible,
    #[postgres(name = "locked")]
    Locked,
    #[postgres(name = "unlocked")]
    Unlocked,
}

pub struct Database {
    pub pool: Pool,
}

pub struct JoinChannel {
    pub id: Id<ChannelMarker>,
    pub guild_id: Id<GuildMarker>,
    pub access_role_id: Option<Id<RoleMarker>>,
    pub parent_id: Option<Id<ChannelMarker>>,
    pub permanence: bool,
    pub privacy: ChannelPrivacy,
}

pub struct VoiceChannel {
    pub id: Id<ChannelMarker>,
    pub guild_id: Id<GuildMarker>,
    pub owner_id: Option<Id<UserMarker>>,
    pub permanence: bool,
    pub privacy: ChannelPrivacy,
}

impl Database {
    pub async fn create_tables(&self) -> Result<()> {
        let client = self.pool.get().await?;
        let statement = "
            create extension if not exists \"uuid-ossp\";

            -- privacy_state enum
            do $$
            begin
                create type channel_privacy as enum (
                    'invisible',
                    'locked',
                    'unlocked'
                );
            exception
                when duplicate_object then null;
            end $$;

            -- join_channel table
            create table if not exists public.join_channel (
                id int8 primary key,
                guild_id int8 not null,
                access_role_id int8,
                parent_id int8,
                permanence boolean not null default false,
                privacy channel_privacy not null default 'unlocked'
            );

            -- voice_channel table
            create table if not exists public.voice_channel (
                id int8 primary key,
                guild_id int8 not null,
                owner_id int8,
                permanence boolean not null,
                privacy channel_privacy not null
            );

            create index if not exists join_channel_guild_id_idx on join_channel(guild_id);
            create index if not exists join_channel_access_role_id_idx on join_channel(access_role_id);
            create index if not exists voice_channel_guild_id_idx on voice_channel(guild_id);
        ";

        client
            .batch_execute(statement)
            .await
            .wrap_err("I'm unable to create tables.")?;

        Ok(())
    }

    pub async fn guild_join_channels(&self, guild_id: Id<GuildMarker>) -> Result<Vec<JoinChannel>> {
        let client = self.pool.get().await?;
        let rows_result = client
            .query(
                "
                    select
                        *
                    from
                        join_channel
                    where
                        guild_id = $1;
                ",
                &[&(guild_id.get() as i64)],
            )
            .await
            .wrap_err("I'm unable to run the \"guild_join_channels\" endpoint.");
        let join_channels = rows_result
            .unwrap_or_default()
            .into_iter()
            .map(|row| JoinChannel::from(row))
            .collect::<Vec<JoinChannel>>();

        Ok(join_channels)
    }

    pub async fn guild_voice_channels(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<VoiceChannel>> {
        let client = self.pool.get().await?;
        let rows_result = client
            .query(
                "
                    select
                        *
                    from
                        voice_channel
                    where
                        guild_id = $1;
                ",
                &[&(guild_id.get() as i64)],
            )
            .await
            .wrap_err("I'm unable to run the \"guild_voice_channels\" endpoint.");
        let voice_channels = rows_result
            .unwrap_or_default()
            .into_iter()
            .map(|row| VoiceChannel::from(row))
            .collect::<Vec<VoiceChannel>>();

        Ok(voice_channels)
    }

    pub async fn insert_join_channel(
        &self,
        id: Id<ChannelMarker>,
        guild_id: Id<GuildMarker>,
        access_role_id: Option<Id<RoleMarker>>,
        parent_id: Option<Id<ChannelMarker>>,
        permanence: bool,
        privacy: ChannelPrivacy,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    insert into
                        join_channel
                    values
                        ($1, $2, $3, $4, $5, $6)
                    on conflict
                    do nothing;
                ",
                &[
                    &(id.get() as i64),
                    &(guild_id.get() as i64),
                    &access_role_id.map(|access_role_id| access_role_id.get() as i64),
                    &parent_id.map(|parent_id| parent_id.get() as i64),
                    &permanence,
                    &privacy,
                ],
            )
            .await
            .wrap_err("I'm unable to run the \"insert_join_channel\" endpoint.")?;

        Ok(())
    }

    pub async fn insert_voice_channel(
        &self,
        id: Id<ChannelMarker>,
        guild_id: Id<GuildMarker>,
        owner_id: Option<Id<UserMarker>>,
        permanence: bool,
        privacy: ChannelPrivacy,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    insert into
                        voice_channel
                    values
                        ($1, $2, $3, $4, $5)
                    on conflict
                    do nothing;
                ",
                &[
                    &(id.get() as i64),
                    &(guild_id.get() as i64),
                    &owner_id.map(|owner_id| owner_id.get() as i64),
                    &permanence,
                    &privacy,
                ],
            )
            .await
            .wrap_err("I'm unable to run the \"insert_voice_channel\" endpoint.")?;

        Ok(())
    }

    pub fn new() -> Self {
        Self {
            pool: Pool::builder(Manager::from_config(
                Config::from_str(DATABASE_URL.as_str()).unwrap(),
                NoTls,
                ManagerConfig {
                    recycling_method: RecyclingMethod::Fast,
                },
            ))
            .max_size(16)
            .build()
            .wrap_err("Unable to create connection pool.")
            .unwrap(),
        }
    }

    pub async fn remove_guild(&self, id: Id<GuildMarker>) -> Result<()> {
        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let params: &[&(dyn ToSql + Sync)] = &[&(id.get() as i64)];

        transaction
            .execute(
                "
                    delete from
                        join_channel
                    where
                        guild_id = $1;
                ",
                params,
            )
            .await
            .wrap_err("I'm unable to run the first query of the \"remove_guild\" endpoint.")?;
        transaction
            .execute(
                "
                    delete from
                        voice_channel
                    where
                        guild_id = $1;
                ",
                params,
            )
            .await
            .wrap_err("I'm unable to run the second query of the \"remove_guild\" endpoint.")?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn remove_join_channel(&self, id: Id<ChannelMarker>) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    delete from
                        join_channel
                    where
                        id = $1;
                ",
                &[&(id.get() as i64)],
            )
            .await
            .wrap_err("I'm unable to run the \"remove_join_channel\" endpoint.")?;

        Ok(())
    }

    pub async fn remove_unknown_channels(
        &self,
        channel_ids: Vec<Id<ChannelMarker>>,
        guild_id: Id<GuildMarker>,
    ) -> Result<()> {
        let mut client = self.pool.get().await?;
        let transaction = client.transaction().await?;
        let params: &[&(dyn ToSql + Sync)] = &[
            &channel_ids
                .into_iter()
                .map(|id| id.get() as i64)
                .collect::<Vec<i64>>(),
            &(guild_id.get() as i64),
        ];

        transaction
            .execute(
                "
                    delete from
                        join_channel
                    where
                        not(id = any($1::int8[]))
                        and guild_id = $2;
                ",
                params,
            )
            .await
            .wrap_err(
                "I'm unable to run the first query of the \"remove_unknown_channels\" endpoint.",
            )?;
        transaction
            .execute(
                "
                    delete from
                        voice_channel
                    where
                        not(id = any($1::int8[]))
                        and guild_id = $2;
                ",
                params,
            )
            .await
            .wrap_err(
                "I'm unable to run the second query of the \"remove_unknown_channels\" endpoint.",
            )?;

        transaction.commit().await?;

        Ok(())
    }

    pub async fn remove_voice_channel(&self, id: Id<ChannelMarker>) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    delete from
                        voice_channel
                    where
                        id = $1;
                ",
                &[&(id.get() as i64)],
            )
            .await
            .wrap_err("I'm unable to run the \"remove_voice_channel\" endpoint.")?;

        Ok(())
    }

    pub async fn update_join_channel_access_role_id(
        &self,
        id: Id<ChannelMarker>,
        access_role_id: Option<Id<RoleMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        join_channel
                    set
                        access_role_id = $2
                    where
                        id = $1;
                ",
                &[
                    &(id.get() as i64),
                    &access_role_id.map(|access_role_id| access_role_id.get() as i64),
                ],
            )
            .await
            .wrap_err("I'm unable to run the \"update_join_channel_access_role_id\" endpoint.")?;

        Ok(())
    }

    pub async fn update_join_channel_parent_id(
        &self,
        id: Id<ChannelMarker>,
        parent_id: Option<Id<ChannelMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        join_channel
                    set
                        parent_id = $2
                    where
                        id = $1;
                ",
                &[
                    &(id.get() as i64),
                    &parent_id.map(|parent_id| parent_id.get() as i64),
                ],
            )
            .await
            .wrap_err("I'm unable to run the \"update_join_channel_parent_id\" endpoint.")?;

        Ok(())
    }

    pub async fn update_join_channel_permanence(
        &self,
        id: Id<ChannelMarker>,
        permanence: bool,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        join_channel
                    set
                        permanence = $2
                    where
                        id = $1;
                ",
                &[&(id.get() as i64), &permanence],
            )
            .await
            .wrap_err("I'm unable to run the \"update_join_channel_permanence\" endpoint.")?;

        Ok(())
    }

    pub async fn update_join_channel_privacy(
        &self,
        id: Id<ChannelMarker>,
        privacy: ChannelPrivacy,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        join_channel
                    set
                        privacy = $2
                    where
                        id = $1;
                ",
                &[&(id.get() as i64), &privacy],
            )
            .await
            .wrap_err("I'm unable to run the \"update_join_channel_privacy\" endpoint.")?;

        Ok(())
    }

    pub async fn update_voice_channel_owner_id(
        &self,
        id: Id<ChannelMarker>,
        owner_id: Option<Id<UserMarker>>,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        voice_channel
                    set
                        owner_id = $2
                    where
                        id = $1;
                ",
                &[
                    &(id.get() as i64),
                    &owner_id.map(|owner_id| owner_id.get() as i64),
                ],
            )
            .await
            .wrap_err("I'm unable to run the \"update_voice_channel_owner_id\" endpoint.")?;

        Ok(())
    }

    pub async fn update_voice_channel_permanence(
        &self,
        id: Id<ChannelMarker>,
        permanence: bool,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        voice_channel
                    set
                        permanence = $2
                    where
                        id = $1;
                ",
                &[&(id.get() as i64), &permanence],
            )
            .await
            .wrap_err("I'm unable to run the \"update_voice_channel_permanence\" endpoint.")?;

        Ok(())
    }

    pub async fn update_voice_channel_privacy(
        &self,
        id: Id<ChannelMarker>,
        privacy: ChannelPrivacy,
    ) -> Result<()> {
        let client = self.pool.get().await?;

        client
            .execute(
                "
                    update
                        voice_channel
                    set
                        privacy = $2
                    where
                        id = $1;
                ",
                &[&(id.get() as i64), &privacy],
            )
            .await
            .wrap_err("I'm unable to run the \"update_voice_channel_privacy\" endpoint.")?;

        Ok(())
    }
}

impl From<Row> for JoinChannel {
    fn from(row: Row) -> Self {
        Self {
            id: Id::new(row.get::<_, i64>("id") as u64),
            guild_id: Id::new(row.get::<_, i64>("guild_id") as u64),
            access_role_id: row
                .get::<_, Option<i64>>("access_role_id")
                .map(|id| Id::new(id as u64)),
            parent_id: row
                .get::<_, Option<i64>>("parent_id")
                .map(|id| Id::new(id as u64)),
            permanence: row.get::<_, bool>("permanence"),
            privacy: row.get::<_, ChannelPrivacy>("privacy"),
        }
    }
}

impl From<Row> for VoiceChannel {
    fn from(row: Row) -> Self {
        Self {
            id: Id::new(row.get::<_, i64>("id") as u64),
            guild_id: Id::new(row.get::<_, i64>("guild_id") as u64),
            owner_id: row
                .get::<_, Option<i64>>("owner_id")
                .map(|id| Id::new(id as u64)),
            permanence: row.get::<_, bool>("permanence"),
            privacy: row.get::<_, ChannelPrivacy>("privacy"),
        }
    }
}
