use crate::{Context, Error, RiotKey};
use serde::Deserialize;

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

#[derive(Debug, poise::ChoiceParameter)]
pub enum LeagueCommandChoice {
    #[name = "Account Statistics"]
    #[name = "acc"]
    Account,
    #[name = "Match History"]
    #[name = "mh"]
    MatchHistory,
}

/// LEAGUE STATS
#[poise::command(slash_command, prefix_command)]
pub async fn lol(
    ctx: Context<'_>,
    #[description = "WHAT DO YOU WANT TO CHECK?"] cmd: LeagueCommandChoice,
    #[description = "GIVE ME USER#TAG"] username: String,
) -> Result<(), Error> {
    ctx.say(format!("{:?}", cmd)).await?;

    // let riot_client = {
    //     let data = ctx.serenity_context().data.read().await;
    //     data.get::<RiotKey>()
    //         .cloned()
    //         .expect("Should be in typemap")
    // };

    let riot_client = ctx.data().riot_client.clone();

    match cmd {
        LeagueCommandChoice::Account => {
            let usr_vec: Vec<&str> = username.split("#").collect();
            if let [game_name, tag_line] = usr_vec.as_slice() {
                let account_http = format!(
                    "https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}",
                    game_name, tag_line
                );
                let acc_res = riot_client.get(account_http.as_str()).send().await?;
                let acc_info = acc_res.json::<AccountInfo>().await?;
                let acc_puuid = acc_info.puuid;

                let account_lol_http = format!(
                    "https://na1.api.riotgames.com/lol/league/v4/entries/by-puuid/{}",
                    acc_puuid
                );
                let acc_lol_res = riot_client.get(account_lol_http.as_str()).send().await?;
                let acc_lol_info = acc_lol_res.json::<LeagueAccountStatus>().await?;

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

                ctx.say(format!("
**{username}** IS RANKED
**{solo_tier} {solo_div} {solo_lp}LP** IN RANKED SOLO/DUO WITH **{solo_w}** WINS AND **{solo_l}** LOSSES
**{flex_tier} {flex_div} {flex_lp}LP** IN RANKED FLEX WITH **{flex_w}** WINS AND **{flex_l}** LOSSES
:robot::nerd:
                    ").as_str()).await?;
            } else {
                return Err("Username in incorrect format!".into());
            }
        }
        LeagueCommandChoice::MatchHistory => {
            ctx.say("THIS COMMAND IS NOT IMPLEMENTED YET... :robot::cry:")
                .await?;
        }
    }
    Ok(())
    // let lol_vec: Vec<&str> = msg.content.split_ascii_whitespace().collect();
    // match lol_vec.as_slice() {
    //     [_lol] => {
    //         botsay(
    //             ctx,
    //             &msg,
    //             "NO COMMAND DETECTED ... USE **acc** OR **mh** :robot::anger:",
    //         )
    //         .await
    //     }
    //     [_lol, cmd] => match *cmd {
    //         "acc" | "mh" => {
    //             botsay(
    //                 ctx,
    //                 &msg,
    //                 format!(
    //                     "NO ACCOUNT DETECTED FOR COMMAND: **{}** ... NEEDS USERNAME#TAG",
    //                     cmd
    //                 )
    //                 .as_str(),
    //             )
    //             .await
    //         }
    //         _ => {
    //             botsay(
    //                 ctx,
    //                 &msg,
    //                 "I DO NOT RECOGNIZE THIS COMMAND :robot::confused:",
    //             )
    //             .await
    //         }
    //     },
    //     [lol, cmd, ..] => {
    //         let usr = msg.content.clone().split_off(lol.len() + cmd.len() + 2);
    // let usr_vec: Vec<&str> = usr.split("#").collect();
    // if let [game_name, tag_line] = usr_vec.as_slice() {
    //     match *cmd {
    //         "acc" => {
    //             let account_http = format!("https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/{}/{}", game_name, tag_line);
    //             match self.riot_client.get(account_http.as_str()).send().await {
    //                 Err(why) => {
    //                     error!("Error sending get request to {}: {why:?}", account_http)
    //                 }
    //                 Ok(acc_res) => match acc_res.json::<AccountInfo>().await {
    //                     Err(why) => {
    //                         error!("Error converting response to text: {why:?}")
    //                     }
    //                     Ok(acc_info) => {
    //                         let acc_puuid = acc_info.puuid;
    //                         let account_lol_http = format!("https://na1.api.riotgames.com/lol/league/v4/entries/by-puuid/{}", acc_puuid);
    //                         match self
    //                             .riot_client
    //                             .get(account_lol_http.as_str())
    //                             .send()
    //                             .await
    //                         {
    //                             Err(why) => {
    //                                 error!(
    //                                     "Error sending get request to {}: {why:?}",
    //                                     account_lol_http
    //                                 )
    //                             }
    //                             Ok(acc_lol_res) => {
    //                                 match acc_lol_res.json::<LeagueAccountStatus>().await {
    //                                     Err(why) => error!(
    //                                         "Error converting response to text: {why:?}"
    //                                     ),
    //                                     Ok(acc_lol_info) => {
    //                                         let solo = acc_lol_info.solo;
    //                                         let flex = acc_lol_info.flex;
    //
    //                                         let solo_tier = solo.tier;
    //                                         let solo_div = solo.rank;
    //                                         let solo_lp = solo.league_points;
    //                                         let solo_w = solo.wins;
    //                                         let solo_l = solo.losses;
    //
    //                                         let flex_tier = flex.tier;
    //                                         let flex_div = flex.rank;
    //                                         let flex_lp = flex.league_points;
    //                                         let flex_w = flex.wins;
    //                                         let flex_l = flex.losses;
    //
    //                                         botsay(ctx, &msg, format!("**{usr}** IS RANKED\r\n**{solo_tier} {solo_div} {solo_lp}LP** IN RANKED SOLO/DUO WITH **{solo_w}** WINS AND **{solo_l}** LOSSES\r\n**{flex_tier} {flex_div} {flex_lp}LP** IN RANKED FLEX WITH **{flex_w}** WINS AND **{flex_l}** LOSSES\r\n :robot::nerd:").as_str()).await;
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 },
    //             }
    //         }
    //         _ => {
    //             botsay(
    //                 ctx,
    //                 &msg,
    //                 "I DO NOT RECOGNIZE THIS COMMAND :robot::confused:",
    //             )
    //             .await
    //         }
    //     }
    // } else {
    //     botsay(ctx, &msg, "USERNAME IN INCORRECT FORMAT :robot::anger:").await
    // }
    //     }
    //     _ => {
    //         botsay(
    //             ctx,
    //             &msg,
    //             "I DO NOT RECOGNIZE THIS COMMAND :robot::confused:",
    //         )
    //         .await
    //     }
    // }
}
