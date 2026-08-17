use crate::{Context, Error};

use jiff::civil::date;
use jiff::{Unit, Zoned};

use rand::prelude::*;

/// Eileen!
#[poise::command(slash_command, prefix_command)]
pub async fn eileen(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("HELLO EILEEN :goat::saluting_face::robot:").await?;

    let now = Zoned::now().round(Unit::Second)?;
    let now_ny = now.in_tz("America/New_York")?;
    let bd_year;

    match (now_ny.month(), now_ny.day()) {
        (8, ..18) => bd_year = now_ny.year(),
        (8, 18..) => bd_year = now_ny.year() + 1,
        (9.., _) => bd_year = now_ny.year() + 1,
        (..8, _) => bd_year = now_ny.year(),
    }

    let bd_ny = date(bd_year, 8, 17).in_tz("America/New_York")?;

    match (now_ny.month(), now_ny.day()) {
        (8, 17) => {
            sing(ctx).await?;
        }
        _ => {
            let span = now_ny.until((Unit::Month, &bd_ny))?;
            ctx.say(format!(
                "THERE ARE {span:#} UNTIL EILEEN'S BIRTHDAY :birthday::robot:"
            ))
            .await?;
        }
    }

    Ok(())
}

async fn sing(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(
        "IT IS EILEEN'S BIRTHDAY! :goat::balloon::robot:\n\
            TO CELEBRATE, I HAVE PREPARED A SONG:",
    )
    .await?;

    ctx.say(
        "HAPPY BIRTHDAY TO YOU :tada:\n\
        HAPPY BIRTHDAY TO YOU :tada:\n\
        HAPPY BIRTHDAY DEAR EILEEN :tada:\n\
        HAPPY BIRTHDAY TO YOU :tada:\n\
        :birthday::robot::partying_face:",
    )
    .await?;

    Ok(())
}

/// Am I alive?
#[poise::command(slash_command, prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!(
        "PONG! :ping_pong:\n{:?} MS TO HIT  ... :stopwatch::robot:",
        ctx.ping().await.as_millis()
    ))
    .await?;
    Ok(())
}

/// It's time to gamble...
#[poise::command(slash_command, prefix_command)]
pub async fn roll(ctx: Context<'_>, die: u32) -> Result<(), Error> {
    let rnd = {
        let mut rng = rand::rng();
        rng.random_range(0..die)
    };

    ctx.say(format!("YOU ROLLED A **{}** :robot::game_die:", rnd).as_str())
        .await?;

    Ok(())
}

/// What else can I say?
#[poise::command(slash_command, prefix_command)]
pub async fn kys(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("KILL YOURSELF NOW! :robot::joy:").await?;

    Ok(())
}
