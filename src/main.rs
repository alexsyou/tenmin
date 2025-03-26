use std::env;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::EditMessage;
use serenity::model::channel::Embed;
use serenity::model::channel::Message;
use serenity::prelude::*;

use tokio::time::interval;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "!ping" => match msg.channel_id.say(&ctx.http, "Pong!").await {
                Err(why) => println!("Error sending message: {why:?}"),
                Ok(_) => println!("Sent message!"),
            },
            x if x.starts_with("!time") => {
                let time_vec: Vec<&str> = x.split(' ').collect();
                if let Ok(dur) = time_vec[1].trim().parse::<u32>() {
                    match msg.channel_id.say(&ctx.http, dur.to_string()).await {
                        Err(why) => println!("Error setting up timer: {why:?}"),
                        Ok(timer_msg) => timeset(ctx, timer_msg, dur).await,
                    };
                }
            }
            _ => {}
        }
    }
}

async fn timeset(ctx: Context, msg: Message, dur: u32) {
    let mut intv = interval(Duration::from_millis(60000));

    match msg
        .channel_id
        .say(
            &ctx.http,
            format!("Setting timer for {} minutes :robot:", dur),
        )
        .await
    {
        Err(why) => println!("Error sending initial timer message: {why:?}"),
        Ok(_) => println!("Sent initial timer message"),
    }

    match msg
        .channel_id
        .say(
            &ctx.http,
            format!("{}:00 MINUTES REMAINING ... :bomb::robot:", dur),
        )
        .await
    {
        Err(why) => println!("Error sending countdown message: {why:?}"),
        Ok(ctdwn) => {
            println!("Sent countdown message");
            intv.tick().await;
            for i in 1..=dur {
                intv.tick().await;
                let edit = match i {
                    x if x == dur => EditMessage::new()
                        .content(":alarm_clock: TIMER COMPLETE. :alarm_clock: :boom::robot:"),
                    _ => EditMessage::new().content(format!(
                        "{}:00 MINUTES REMIAINING... :bomb::robot:",
                        dur - i
                    )),
                };
                match ctdwn
                    .channel_id
                    .edit_message(&ctx.http, &ctdwn.id, edit)
                    .await
                {
                    Err(why) => println!("Error editing message: {why:?}"),
                    Ok(edit_msg) => {
                        let cont = edit_msg.content;
                        println!("Edited message to: {cont:?}");
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected token from environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
