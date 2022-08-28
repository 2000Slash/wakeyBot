use std::net::Ipv4Addr;
use std::process::Command;

use log::{debug, error, info, warn};

use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::prelude::*;

use crate::discord::Ip;
use crate::{decode_hex, fetch_ip};

#[group]
#[owners_only]
#[only_in(dm)]
#[commands(ip, wake, ping)]
pub struct General;

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
        msg.reply(&ctx, String::from_utf8(output.unwrap().stdout).unwrap())
            .await?;
    } else {
        let error_string = String::from_utf8(output.unwrap().stdout).unwrap();
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
async fn ip(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut force = false;
    if args.rest().eq("force") {
	info!("Using force to update ip.");
	force = true;
    }
    // Send the currently saved Ip to the user. If IP is none, try to fetch it again
    let mut ip: Option<Ipv4Addr>;
    {
        let data = ctx.data.read().await;
        let ip_lock = data.get::<Ip>().unwrap().clone();
        ip = *ip_lock.read().await;
    }
    info!("Saved ip: {:?}", ip);
    if ip.is_none() || force {
        info!("Fetching new ip...");
        ip = fetch_ip();
        {
            let data = ctx.data.write().await;
            let ip_lock = data.get::<Ip>().unwrap().clone();
            let mut writer = ip_lock.write().await;
            *writer = ip;
            info!("New ip is {:?}", ip);
        }
    }
    info!("Current ip: {:?}", ip);
    msg.reply(ctx, format!("The ip address is: {:?}", ip))
        .await?;
    Ok(())
}
