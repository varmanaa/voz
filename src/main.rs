mod events;
mod interactions;
mod structs;
mod utilities;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use events::handle_event;
use eyre::Result;
use structs::context::Context;
use tokio::signal;

use twilight_gateway::{
    create_recommended, CloseFrame, Config as TwilightGatewayConfig, Event, Shard, StreamExt,
};
use twilight_http::Client;
use utilities::constants::{DISCORD_TOKEN, INTENTS, WANTED_EVENT_TYPES};

static SHUTDOWN: AtomicBool = AtomicBool::new(false);

async fn runner(mut shard: Shard, context: Arc<Context>) -> Result<()> {
    while let Some(item) = shard.next_event(*WANTED_EVENT_TYPES).await {
        match item {
            Ok(Event::GatewayClose(_)) if SHUTDOWN.load(Ordering::Relaxed) => break,
            Ok(event) => {
                let event_context = Arc::clone(&context);

                tokio::spawn(async move { handle_event(event_context, event).await.unwrap() })
            }
            Err(_source) => {
                continue;
            }
        };
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let client = Client::new(DISCORD_TOKEN.to_owned());
    let application_id = client.current_user_application().await?.model().await?.id;
    let config = TwilightGatewayConfig::new(DISCORD_TOKEN.to_owned(), *INTENTS);
    let shards = create_recommended(&client, config, |_, builder| builder.build()).await?;
    let shard_count = shards.len();
    let mut senders = Vec::with_capacity(shard_count);
    let mut tasks = Vec::with_capacity(shard_count);
    let context = Arc::new(Context::new(application_id, client));

    context.database.create_tables().await?;

    #[cfg(feature = "set-global-commands")]
    context
        .interaction_client()
        .set_global_commands(&utilities::constants::COMMANDS)
        .await?;

    for shard in shards {
        let shard_context = Arc::clone(&context);

        senders.push(shard.sender());
        tasks.push(tokio::spawn(runner(shard, shard_context)))
    }

    signal::ctrl_c().await?;
    SHUTDOWN.store(true, Ordering::Relaxed);

    for sender in senders {
        _ = sender.close(CloseFrame::NORMAL);
    }

    for join_handle in tasks {
        _ = join_handle.await;
    }

    Ok(())
}
