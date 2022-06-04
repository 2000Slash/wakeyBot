use std::collections::HashSet;
use std::env;
use std::net::Ipv4Addr;
use std::sync::Arc;

use log::{error, info};

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

use crate::commands::GENERAL_GROUP;

/// holds a single IPv4Addr or None

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

pub struct Ip;

impl TypeMapKey for Ip {
    type Value = Arc<RwLock<Option<Ipv4Addr>>>;
}

/// Starts the discord bot and returns it
pub async fn create_client() -> Client {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::DIRECT_MESSAGES;

    let mut owners: HashSet<UserId> = HashSet::new();
    owners.insert(UserId(220975319786586112));

    let framework = StandardFramework::new()
        .configure(|c| c.with_whitespace(true).prefix("!").owners(owners))
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Ip>(Arc::new(RwLock::new(None)));
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
    client
}
