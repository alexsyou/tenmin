use std::env;
use std::time::Duration;

use anyhow::Context as _;

use serenity::async_trait;
use serenity::builder::CreateCommand;
use serenity::builder::EditMessage;
use serenity::model::application::Command;
use serenity::model::channel::Embed;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ApplicationId;
use serenity::prelude::*;

use shuttle_runtime::SecretStore;

use tracing::{error, info};

use tokio::time::interval;

use rand::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.split_ascii_whitespace().collect::<Vec<&str>>()[0] {
            "!ping" => match msg.channel_id.say(&ctx.http, "Pong!").await {
                Err(why) => error!("Error sending message: {why:?}"),
                Ok(_) => info!("Sent message!"),
            },
            "!time" => {
                let time_vec: Vec<&str> = msg.content.split_ascii_whitespace().collect();
                match time_vec.len() {
                    1 => botsay(ctx, &msg, "NO TIME DETECTED :robot::anger:").await,
                    2 => {
                        if let Ok(dur) = time_vec[1].trim().parse::<u32>() {
                            timeset(ctx, &msg, dur).await;
                        } else {
                            botsay(ctx, &msg, "TIME IN INCORRECT FORMAT :robot::anger:").await
                        }
                    }
                    3 => {
                        if let (Ok(dur), title) = (time_vec[1].trim().parse::<u32>(), time_vec[2]) {
                            timeset(ctx, &msg, dur).await;
                        } else {
                            botsay(ctx, &msg, "TIME IN INCORRECT FORMAT :robot::anger:").await
                        }
                    }
                    _ => {
                        botsay(
                            ctx,
                            &msg,
                            "I DO NOT RECOGNIZE THIS PATTERN :robot::confused:",
                        )
                        .await
                    }
                }
            }
            "!kys" => {
                botsay(ctx, &msg, "KILL YOURSELF NOW! :robot::joy:").await;
                //
                // let random = rand::random::<u8>();
                // match random {
                //     0..128 => botsay(ctx, &msg, "KILL YOURSELF NOW! :robot::joy:").await,
                //     _ => {
                //         botsay(
                //             ctx,
                //             &msg,
                //             "KEEP YOURSELF SAFE TONIGHT. :gun::robot::slight_smile:",
                //         )
                //         .await
                //     }
                // }
            }
            "!eileen" => {
                botsay(ctx, &msg, "HELLO EILEEN :goat::saluting_face::robot:").await;
            }
            "!roll" => {
                let roll_vec: Vec<&str> = msg.content.split_ascii_whitespace().collect();
                match roll_vec.len() {
                    1 => botsay(ctx, &msg, "NO TIME DETECTED :robot::anger:").await,
                    2 => {
                        if let Ok(die) = roll_vec[1].trim().parse::<u32>() {
                            let rnd = {
                                let mut rng = rand::rng();
                                rng.random_range(0..die)
                            };
                            botsay(
                                ctx,
                                &msg,
                                format!("YOU ROLLED A **{}** :robot::game_die:", rnd).as_str(),
                            )
                            .await
                        } else {
                            botsay(ctx, &msg, "ROLL COUNT IN INCORRECT FORMAT :robot::anger:").await
                        }
                    }
                    _ => {
                        botsay(
                            ctx,
                            &msg,
                            "I DO NOT RECOGNIZE THIS PATTERN :robot::confused:",
                        )
                        .await
                    }
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

async fn botsay(ctx: Context, msg: &Message, diag: &str) {
    match msg.channel_id.say(&ctx.http, diag).await {
        Err(why) => error!("Error sending message: {why:?}"),
        Ok(res) => info!("Message sent successfully: {}", res.content),
    }
}

async fn timeset(ctx: Context, msg: &Message, dur: u32) {
    let mut intv = interval(Duration::from_millis(60000));

    match msg
        .channel_id
        .say(
            &ctx.http,
            format!(
                "SETTING TIMER FOR {} MINUTE{} :robot:",
                dur,
                if dur == 1 { "" } else { "S" }
            ),
        )
        .await
    {
        Err(why) => error!("Error sending initial timer message: {why:?}"),
        Ok(_) => info!("Sent initial timer message"),
    }

    match msg
        .channel_id
        .say(
            &ctx.http,
            format!("{}:00 MINUTES REMAINING ... :bomb::robot:", dur),
        )
        .await
    {
        Err(why) => error!("Error sending countdown message: {why:?}"),
        Ok(ctdwn) => {
            info!("Sent countdown message");
            intv.tick().await;
            for i in 1..=dur {
                intv.tick().await;
                let edit = match i {
                    x if x == dur => EditMessage::new()
                        .content(":alarm_clock: TIMER COMPLETE :alarm_clock: :boom::robot:"),
                    _ => EditMessage::new().content(format!(
                        "{}:00 MINUTES REMAINING ... :bomb::robot:",
                        dur - i
                    )),
                };
                match ctdwn
                    .channel_id
                    .edit_message(&ctx.http, &ctdwn.id, edit)
                    .await
                {
                    Err(why) => error!("Error editing message: {why:?}"),
                    Ok(edit_msg) => {
                        let cont = edit_msg.content;
                        info!("Edited message to: {cont:?}");
                    }
                }
            }

            if let Err(why) = msg
                .reply_ping(
                    &ctx.http,
                    "YOUR TIME HAS COME TO AN END :index_pointing_at_the_viewer::robot:",
                )
                .await
            {
                error!("Error replying to timer: {why:?}");
            }
        }
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let builder = CreateCommand::new("ping").description("A simple ping!");
    let _ = Command::create_global_command(&client.http, builder).await;

    Ok(client.into())

    // if let Err(why) = client.start().await {
    //     println!("Client error: {why:?}");
    // }
}
