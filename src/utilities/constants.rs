use std::{env, sync::LazyLock};

use eyre::WrapErr;
use twilight_gateway::{EventTypeFlags, Intents};
use twilight_model::{
    application::command::{Command, CommandType},
    channel::ChannelType,
    guild::Permissions,
};
use twilight_util::builder::command::{
    BooleanBuilder, ChannelBuilder, CommandBuilder, IntegerBuilder, RoleBuilder, StringBuilder,
    SubCommandBuilder, UserBuilder,
};

pub static COMMANDS: LazyLock<Vec<Command>> = LazyLock::new(|| {
    vec![
        CommandBuilder::new("join", "Modify join channels", CommandType::ChatInput)
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .option(
                SubCommandBuilder::new("access-role", "Modify the access role for a join channel")
                    .option(
                        StringBuilder::new("channel", "The join channel")
                            .autocomplete(true)
                            .required(true)
                            .build(),
                    )
                    .option(
                        RoleBuilder::new("role", "The role to let access the join channel").build(),
                    )
                    .build(),
            )
            .option(
                SubCommandBuilder::new(
                    "category",
                    "Modify the category (to create voice channels under)",
                )
                .option(
                    StringBuilder::new("channel", "The join channel")
                        .autocomplete(true)
                        .required(true)
                        .build(),
                )
                .option(
                    ChannelBuilder::new("category", "The category to create voice channels under")
                        .channel_types(vec![ChannelType::GuildCategory])
                        .build(),
                )
                .build(),
            )
            .option(
                SubCommandBuilder::new("create", "Create a join channel")
                    .option(
                        StringBuilder::new("name", "The name")
                            .max_length(50)
                            .min_length(1)
                            .required(true)
                            .build(),
                    )
                    .option(
                        RoleBuilder::new("access-role", "The role to let access the join channel")
                            .build(),
                    )
                    .option(
                        ChannelBuilder::new(
                            "category",
                            "The category to create voice channels under",
                        )
                        .channel_types(vec![ChannelType::GuildCategory])
                        .build(),
                    )
                    .option(
                        BooleanBuilder::new(
                            "permanence",
                            "Should created voice channels remain when empty?",
                        )
                        .build(),
                    )
                    .option(
                        StringBuilder::new(
                            "privacy",
                            "The default privacy level for created voice channels",
                        )
                        .choices(vec![
                            ("Invisible", "invisible"),
                            ("Locked (and visible)", "locked"),
                            ("Unlocked (and visible)", "unlocked"),
                        ])
                        .build(),
                    )
                    .build(),
            )
            .option(
                SubCommandBuilder::new("name", "Modify the name of a join channel")
                    .option(
                        StringBuilder::new("channel", "The join channel")
                            .autocomplete(true)
                            .required(true)
                            .build(),
                    )
                    .option(
                        StringBuilder::new("name", "The name")
                            .required(true)
                            .build(),
                    )
                    .build(),
            )
            .option(
                SubCommandBuilder::new(
                    "permanence",
                    "Modify the permanence value of a join channel",
                )
                .option(
                    StringBuilder::new("channel", "The join channel")
                        .autocomplete(true)
                        .required(true)
                        .build(),
                )
                .option(
                    BooleanBuilder::new("value", "Should voice channels remain when empty?")
                        .required(true)
                        .build(),
                )
                .build(),
            )
            .option(
                SubCommandBuilder::new(
                    "privacy",
                    "Modify the privacy level of a join channel",
                )
                .option(
                    StringBuilder::new("channel", "The join channel")
                        .autocomplete(true)
                        .required(true)
                        .build(),
                )
                .option(
                    StringBuilder::new("level", "The default privacy level for created voice channels")
                        .choices(vec![
                            ("Invisible", "invisible"),
                            ("Locked (and visible)", "locked"),
                            ("Unlocked (and visible)", "unlocked"),
                        ])
                        .required(true)
                        .build(),
                )
                .build(),
            )
            .option(
                SubCommandBuilder::new("remove", "Remove a join channel")
                    .option(
                        StringBuilder::new("channel", "The join channel")
                            .autocomplete(true)
                            .required(true)
                            .build(),
                    )
                    .build(),
            )
            .option(
                SubCommandBuilder::new("view", "View the current settings of a join channel")
                    .option(
                        StringBuilder::new("channel", "The join channel")
                            .autocomplete(true)
                            .required(true)
                            .build(),
                    )
                    .build(),
            )
            .build(),
        CommandBuilder::new(
            "voice",
            "Modify your voice channel",
            CommandType::ChatInput,
        )
        .option(
            SubCommandBuilder::new("allow-member", "Allow a member permission to join your voice channel")
                .option(
                    UserBuilder::new("member", "The member")
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("bitrate", "Modify the bitrate of your voice channel")
                .option(
                    IntegerBuilder::new("rate", "The bitrate")
                        .max_value(96)
                        .min_value(8)
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(SubCommandBuilder::new("claim", "Claim an unowned voice channel").build())
        .option(SubCommandBuilder::new("delete", "Delete your voice channel").build())
        .option(
            SubCommandBuilder::new("deny-member", "Deny a member permission to join your voice channel")
                .option(
                    UserBuilder::new("member", "The member")
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("name", "Modify the name of your voice channel")
                .option(
                    StringBuilder::new("name", "The name")
                        .max_length(50)
                        .min_length(1)
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("permanence", "Modify the permanence value of your voice channel")
                .option(
                    BooleanBuilder::new("value", "When empty, should your voice channel remain?")
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("privacy", "Modify the privacy level of your voice channel")
                .option(
                    StringBuilder::new("level", "The privacy option")
                        .choices(vec![
                            ("Invisible", "invisible"),
                            ("Locked (and visible)", "locked"),
                            ("Unlocked (and visible)", "unlocked"),
                        ])
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("remove-member", "Remove a member's permission to join your voice channel (and disconnect the member)")
                .option(
                    UserBuilder::new("member", "The member")
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("slow-mode", "Modify the slow mode duration of your voice channel")
                .option(
                    StringBuilder::new("duration", "The slow mode duration")
                        .autocomplete(true)
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("transfer", "Transfer ownership of your voice channel to another member")
                .option(
                    UserBuilder::new("member", "The member")
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("user-limit", "Modify the user limit of your voice channel")
                .option(
                    IntegerBuilder::new("limit", "The limit")
                        .max_value(99)
                        .min_value(0)
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(
            SubCommandBuilder::new("video-quality-mode", "Modify the video quality mode your voice channel")
                .option(
                    StringBuilder::new("mode", "The mode")
                        .choices(vec![("Auto", "auto"), ("720p", "full")])
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .option(SubCommandBuilder::new("view", "View the current settings of your voice channel").build())
        .option(
            SubCommandBuilder::new("voice-region", "Modify the voice region of your voice channel")
                .option(
                    StringBuilder::new("region", "The voice region")
                        .choices(vec![
                            ("Automatic", "automatic"),
                            ("Brazil", "brazil"),
                            ("Hong Kong", "hongkong"),
                            ("India", "india"),
                            ("Japan", "japan"),
                            ("Rotterdam", "rotterdam"),
                            ("Russia", "russia"),
                            ("Singapore", "singapore"),
                            ("South Africa", "southafrica"),
                            ("Sydney", "sydney"),
                            ("US Central", "us-central"),
                            ("US East", "us-east"),
                            ("US South", "us-south"),
                            ("US West", "us-west"),
                        ])
                        .required(true)
                        .build(),
                )
                .build(),
        )
        .build(),
    ]
});

pub static DATABASE_URL: LazyLock<String> = LazyLock::new(|| {
    env::var("DATABASE_URL")
        .wrap_err("Environment variable \"DATABASE_URL\" is not set.")
        .unwrap()
});

pub static DISCORD_TOKEN: LazyLock<String> = LazyLock::new(|| {
    env::var("DISCORD_TOKEN")
        .wrap_err("Environment variable \"DISCORD_TOKEN\" is not set.")
        .unwrap()
});

pub static INTENTS: LazyLock<Intents> =
    LazyLock::new(|| Intents::GUILDS | Intents::GUILD_MEMBERS | Intents::GUILD_VOICE_STATES);

pub static SLOW_MODE_OPTIONS: LazyLock<Vec<[String; 2]>> = LazyLock::new(|| {
    let mut choices = vec![["Off".to_owned(), "0".to_owned()]];

    for seconds in 1..=59u16 {
        let duration = seconds.to_string();

        if seconds.eq(&1) {
            choices.push([format!("{seconds} second"), duration]);
        } else {
            choices.push([format!("{seconds} seconds"), duration]);
        }
    }

    for minutes in 1..=59u16 {
        let duration = (minutes * 60).to_string();

        if minutes.eq(&1) {
            choices.push([format!("{minutes} minute"), duration]);
        } else {
            choices.push([format!("{minutes} minutes"), duration]);
        }
    }

    for hours in 1..=6u16 {
        let duration = (hours * 3_600).to_string();

        if hours.eq(&1) {
            choices.push([format!("{hours} hour"), duration]);
        } else {
            choices.push([format!("{hours} hours"), duration]);
        }
    }

    choices
});

pub static WANTED_EVENT_TYPES: LazyLock<EventTypeFlags> = LazyLock::new(|| {
    EventTypeFlags::CHANNEL_DELETE
        | EventTypeFlags::CHANNEL_UPDATE
        | EventTypeFlags::GUILD_CREATE
        | EventTypeFlags::GUILD_DELETE
        | EventTypeFlags::GUILD_UPDATE
        | EventTypeFlags::INTERACTION_CREATE
        | EventTypeFlags::MEMBER_REMOVE
        | EventTypeFlags::READY
        | EventTypeFlags::ROLE_DELETE
        | EventTypeFlags::UNAVAILABLE_GUILD
        | EventTypeFlags::VOICE_STATE_UPDATE
});
