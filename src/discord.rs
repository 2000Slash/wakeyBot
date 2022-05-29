


use std::net::Ipv4Addr;
use std::env;
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

/// holds a single IPv4Addr or None
struct Ip;

impl TypeMapKey for Ip {
    type Value = Arc<RwLock<Option<Ipv4Addr>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.id != 220975319786586112 {
            return;
        } 
        // Send the currently saved Ip to the user. If IP is none, try to fetch it again
        if msg.content == "!ip" {
            let mut ip: Option<Ipv4Addr>;
            {
                let data = ctx.data.read().await;
                let ip_lock = data.get::<Ip>().unwrap().clone();
                ip = ip_lock.read().await.clone();
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
            if let Err(why) = msg.channel_id.say(&ctx.http, format!("The ip address is: {:?}", ip)).await {
                println!("Error sending message: {:?}", why);
            }
        } else if msg.content == "!wake" {
            // Send a wake on lan packet to a mac address
            let mac_address: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            let magic_packet = wake_on_lan::MagicPacket::new(&mac_address);
            let err = magic_packet.send();
            if err.is_err() {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Could not wake pc").await {
                    println!("Error sending message: {:?}", why);
                }
            } else {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Initializing wakey wakey protocol").await {
                    println!("Error sending message: {:?}", why);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

/// Starts the discord bot and returns it
pub async fn create_client() -> Client {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::DIRECT_MESSAGES;

   let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Ip>(Arc::new(RwLock::new(public_ip::addr_v4().await)));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
    client
}
