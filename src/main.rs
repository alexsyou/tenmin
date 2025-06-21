mod misc;
mod riot_integration;
mod timer;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;

use serenity::async_trait;
use serenity::builder::EditMessage;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use shuttle_runtime::SecretStore;

use tracing::{error, info};

use tokio::task;
use tokio::time::interval;

use rand::prelude::*;

use serde::Deserialize;

//type Data = ();
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct Data {
    riot_client: reqwest::Client,
    yt_client: reqwest::Client,
    timers: Arc<RwLock<Vec<task::JoinHandle<Result<(), Error>>>>>,
}

struct Handler {
    riot_client: reqwest::Client,
}

struct RiotKey;

impl TypeMapKey for RiotKey {
    type Value = reqwest::Client;
}

struct YTKey;

impl TypeMapKey for YTKey {
    type Value = reqwest::Client;
}

#[derive(Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
#[serde(rename_all(deserialize = "camelCase"))]
struct AccountInfo {
    puuid: String,
    //INFO: Forced camelCase by RIOT API.. is this fixable?
    _game_name: String,
    _tag_line: String,
}

#[derive(Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
#[serde(rename_all(deserialize = "camelCase"))]
struct RankInfo {
    _fresh_blood: bool,
    _hot_streak: bool,
    _inactive: bool,
    _league_id: String,
    league_points: u16,
    losses: u16,
    _puuid: String,
    _queue_type: String,
    rank: String,
    _summoner_id: String,
    tier: String,
    _veteran: bool,
    wins: u16,
}

#[derive(Deserialize)]
#[serde(rename_all(serialize = "snake_case"))]
#[serde(rename_all(deserialize = "camelCase"))]
struct LeagueAccountStatus {
    solo: RankInfo,
    flex: RankInfo,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: serenity::client::Context, msg: Message) {
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
                        if let (Ok(dur), _title) = (time_vec[1].trim().parse::<u32>(), time_vec[2])
                        {
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
            "!lol" => {
                let lol_vec: Vec<&str> = msg.content.split_ascii_whitespace().collect();
                match lol_vec.as_slice() {
                    [_lol] => {
                        botsay(
                            ctx,
                            &msg,
                            "NO COMMAND DETECTED ... USE **acc** OR **mh** :robot::anger:",
                        )
                        .await
                    }
                    [_lol, cmd] => match *cmd {
                        "acc" | "mh" => {
                            botsay(
                                ctx,
                                &msg,
                                format!(
                                "NO ACCOUNT DETECTED FOR COMMAND: **{}** ... NEEDS USERNAME#TAG",
                                cmd
                            )
                                .as_str(),
                            )
                            .await
                        }
                        _ => {
                            botsay(
                                ctx,
                                &msg,
                                "I DO NOT RECOGNIZE THIS COMMAND :robot::confused:",
                            )
                            .await
                        }
                    },
                    [lol, cmd, ..] => {
                        let usr = msg.content.clone().split_off(lol.len() + cmd.len() + 2);
                        let usr_vec: Vec<&str> = usr.split("#").collect();
                        if let [game_name, tag_line] = usr_vec.as_slice() {
                            match *cmd {
                                "acc" => {
                                    let account_http = format!("https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}", game_name, tag_line);
                                    match self.riot_client.get(account_http.as_str()).send().await {
                                        Err(why) => {
                                            error!(
                                                "Error sending get request to {}: {why:?}",
                                                account_http
                                            )
                                        }
                                        Ok(acc_res) => match acc_res.json::<AccountInfo>().await {
                                            Err(why) => {
                                                error!("Error converting response to text: {why:?}")
                                            }
                                            Ok(acc_info) => {
                                                let acc_puuid = acc_info.puuid;
                                                let account_lol_http = format!("https://na1.api.riotgames.com/lol/league/v4/entries/by-puuid/{}", acc_puuid);
                                                match self
                                                    .riot_client
                                                    .get(account_lol_http.as_str())
                                                    .send()
                                                    .await
                                                {
                                                    Err(why) => {
                                                        error!(
                                                        "Error sending get request to {}: {why:?}",
                                                        account_lol_http
                                                        )
                                                    }
                                                    Ok(acc_lol_res) => match acc_lol_res.json::<LeagueAccountStatus>().await {
                                                        Err(why) => error!("Error converting response to text: {why:?}"),
                                                        Ok(acc_lol_info) => {
                                                            let solo = acc_lol_info.solo;
                                                            let flex = acc_lol_info.flex;

                                                            let solo_tier = solo.tier;
                                                            let solo_div = solo.rank;
                                                            let solo_lp = solo.league_points;
                                                            let solo_w = solo.wins;
                                                            let solo_l = solo.losses;

                                                            let flex_tier = flex.tier;
                                                            let flex_div = flex.rank;
                                                            let flex_lp = flex.league_points;
                                                            let flex_w = flex.wins;
                                                            let flex_l = flex.losses;




                                                            botsay(ctx, &msg, format!("**{usr}** IS RANKED\r\n**{solo_tier} {solo_div} {solo_lp}LP** IN RANKED SOLO/DUO WITH **{solo_w}** WINS AND **{solo_l}** LOSSES\r\n**{flex_tier} {flex_div} {flex_lp}LP** IN RANKED FLEX WITH **{flex_w}** WINS AND **{flex_l}** LOSSES\r\n :robot::nerd:").as_str()).await;
                                                        }

                                                    }
                                                }
                                            }
                                        },
                                    }
                                }
                                _ => {
                                    botsay(
                                        ctx,
                                        &msg,
                                        "I DO NOT RECOGNIZE THIS COMMAND :robot::confused:",
                                    )
                                    .await
                                }
                            }
                        } else {
                            botsay(ctx, &msg, "USERNAME IN INCORRECT FORMAT :robot::anger:").await
                        }
                    }
                    _ => {
                        botsay(
                            ctx,
                            &msg,
                            "I DO NOT RECOGNIZE THIS COMMAND :robot::confused:",
                        )
                        .await
                    }
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, _: serenity::client::Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

async fn botsay(ctx: serenity::client::Context, msg: &Message, diag: &str) {
    match msg.channel_id.say(&ctx.http, diag).await {
        Err(why) => error!("Error sending message: {why:?}"),
        Ok(res) => info!("Message sent successfully: {}", res.content),
    }
}

async fn timeset(ctx: serenity::client::Context, msg: &Message, dur: u32) {
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

//TODO: Update error handler
async fn error_handler(error: poise::FrameworkError<'_, Data, Error>) {
    poise::builtins::on_error(error).await.unwrap();
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    let disc_token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let riot_token = secrets
        .get("RIOT_TOKEN")
        .context("'RIOT_TOKEN' was not found")?;

    let mut headers = reqwest::header::HeaderMap::new();
    let mut riot_header =
        reqwest::header::HeaderValue::from_str(&riot_token).expect("Err reading RIOT_TOKEN header");
    riot_header.set_sensitive(true);
    headers.insert("X-Riot-Token", riot_header);

    let riot_client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("Err creating reqwest client");

    let yt_client = reqwest::Client::new();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            on_error: |err| Box::pin(error_handler(err)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                ..Default::default()
            },
            commands: vec![
                misc::ping(),
                misc::eileen(),
                misc::kys(),
                misc::roll(),
                riot_integration::lol(),
                timer::time(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                let data = Data {
                    riot_client,
                    yt_client,
                    timers: Arc::new(RwLock::new(Vec::new())),
                };
                Ok(data)
            })
        })
        .build();

    let intents = GatewayIntents::MESSAGE_CONTENT | GatewayIntents::non_privileged();

    //let handle = Handler { riot_client };

    let client = serenity::client::ClientBuilder::new(&disc_token, intents)
        // .event_handler(handle)
        .framework(framework)
        //        .type_map_insert::<RiotKey>(riot_client)
        //        .type_map_insert::<YTKey>(reqwest::Client::new())
        .await
        .expect("Err creating client");

    Ok(client.into())
}
