


use std::collections::HashSet;
use std::net::Ipv4Addr;
use std::env;
use std::num::ParseIntError;
use std::process::Command;
use std::sync::Arc;

use log::{info, error, warn, debug};

use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::framework::standard::{CommandResult, Args};
use serenity::framework::standard::macros::{group, command};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::UserId;
use serenity::prelude::*;

fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// holds a single IPv4Addr or None
struct Ip;

impl TypeMapKey for Ip {
    type Value = Arc<RwLock<Option<Ipv4Addr>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[group]
#[owners_only]
#[only_in(dm)]
#[commands(ip, wake, ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args_str = args.rest();
    info!("Sending ping to {}", args_str);
    let output = Command::new("ping").args(["-c 5", args_str]).output();
    if output.is_err() {
        error!("Could not execute ping command, {:?}", output.err());
        msg.reply(&ctx, "Could not execute ping").await?;
    } else if output.as_ref().unwrap().status.success() {
        info!("ping succeeded.");
        msg.reply(&ctx, String::from_utf8(output.unwrap().stdout).unwrap()).await?;
    } else {
        let error_string = String::from_utf8(output.unwrap().stderr).unwrap();
        error!("Ping unsuccessful: {}", &error_string);
        msg.reply(&ctx, error_string).await?;
    }
    Ok(())
}

#[command]
async fn wake(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // Send a wake on lan packet to a mac address
    let args_str = args.rest();
    let mac_address_vec = decode_hex(args_str);
    if mac_address_vec.is_err() {
        warn!("Could not parse: {}", args_str);
        msg.reply(&ctx, "Could not parse mac").await?;
        return Ok(());
    }
    let mac_address_vec = mac_address_vec.unwrap();
    let mac_address: Result<[u8; 6], _> = mac_address_vec.try_into();
    if mac_address.is_err() {
        warn!("Could not parse: {}", args_str);
        msg.reply(&ctx, "Could not parse mac").await?;
        return Ok(());
    }
    let mac_address = mac_address.unwrap();
    let magic_packet = wake_on_lan::MagicPacket::new(&mac_address);
    let err = magic_packet.send();
    if err.is_err() {
        warn!("Could not wake");
        msg.reply(&ctx, "Could not wake pc").await?;
    } else {
        info!("Waking {}", args_str);
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
    debug!("Saved ip: {:?}", ip);
    if ip.is_none() {
        debug!("Fetching new ip...");
        ip = public_ip::addr_v4().await;
        {
            let data = ctx.data.write().await;
            let ip_lock = data.get::<Ip>().unwrap().clone();
            let mut writer = ip_lock.write().await;
            *writer = ip;
            debug!("New ip is {:?}", ip);
        }
    }
    info!("Current ip: {:?}", ip);
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
        error!("Client error: {:?}", why);
    }
    client
}
