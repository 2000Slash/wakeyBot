


use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{group, command};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

/// holds a single IPv4Addr or None
struct Ip;

impl TypeMapKey for Ip {
    type Value = Arc<RwLock<Option<Ipv4Addr>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[group]
#[owners_only]
#[only_in(dm)]
#[commands(ip, wake)]
struct General;

#[command]
async fn wake(ctx: &Context, msg: &Message) -> CommandResult {
    // Send a wake on lan packet to a mac address
    let mac_address: [u8; 6] = [0, 0, 0, 0, 0, 0];
    let magic_packet = wake_on_lan::MagicPacket::new(&mac_address);
    let err = magic_packet.send();
    if err.is_err() {
        msg.reply(&ctx, "Could not wake pc").await?;
    } else {
        msg.reply(&ctx, "Initializing wakey wakey protocol").await?;
    }

    Ok(())
}

#[command]
async fn ip(ctx: &Context, msg: &Message) -> CommandResult {
    // Send the currently saved Ip to the user. If IP is none, try to fetch it again
    let mut ip: Option<Ipv4Addr>;
    {
        let data = ctx.data.read().await;
        let ip_lock = data.get::<Ip>().unwrap().clone();
        ip = *ip_lock.read().await;
    }
    if ip.is_none() {
        ip = public_ip::addr_v4().await;
        {
            let data = ctx.data.write().await;
            let ip_lock = data.get::<Ip>().unwrap().clone();
            let mut writer = ip_lock.write().await;
            *writer = ip;
        }
    }
    msg.reply(ctx, format!("The ip address is: {:?}", ip)).await?;
    Ok(())
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

    let mut client = Client::builder(&token, intents).event_handler(Handler).framework(framework).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Ip>(Arc::new(RwLock::new(public_ip::addr_v4().await)));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    client
}
