use crate::{Context, Error};

use poise::serenity_prelude::EditMessage;
use poise::serenity_prelude::Mentionable;
use poise::serenity_prelude::User;

use tokio::time::{interval, Duration};

/// DOES ANYONE KNOW HOW LONG THIS WILL TAKE?
#[poise::command(slash_command, prefix_command)]
pub async fn time(
    ctx: Context<'_>,
    #[description = "HOW LONG IS THIS TIMER?"] duration: u32,
    #[description = "WHO SHOULD I BOTHER..."] user: Option<User>,
) -> Result<(), Error> {
    let time_msg_handle = ctx
        .say(format!(
            "SETTING TIMER FOR {} SECOND{} :robot:",
            duration,
            if duration == 1 { "" } else { "S" }
        ))
        .await?;

    let time_msg = time_msg_handle.into_message().await?;
    let time_channel_id = time_msg.channel_id;
    let time_message_id = time_msg.id;
    let time_http = ctx.serenity_context().http.clone();

    ctx.data()
        .timers
        .write()
        .await
        .push(tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            let mut time_rem = duration;

            loop {
                if time_rem == 0 {
                    break;
                }
                interval.tick().await;
                let new_time = time_rem;
                time_channel_id
                    .edit_message(
                        &time_http,
                        time_message_id,
                        EditMessage::new().content(format!(
                            "{} SECOND{} REMAINING ... :bomb::robot:",
                            new_time,
                            if new_time == 1 { "" } else { "S" }
                        )),
                    )
                    .await?;
                // time_msg_handle
                //     .edit(
                //         ctx,
                //         CreateReply::default().content(format!(
                //             "{} SECOND{} REMAINING ... :bomb::robot:",
                //             new_time,
                //             if new_time == 1 { "" } else { "S" }
                //         )),
                //     )
                //     .await;
                time_rem -= 1;

                if time_rem == 0 {
                    break;
                }
            }

            time_channel_id
                .edit_message(
                    &time_http,
                    time_message_id,
                    EditMessage::new()
                        .content(":alarm_clock: TIMER COMPLETE :alarm_clock: :boom::robot:"),
                )
                .await?;

            time_msg
                .reply_ping(
                    &time_http,
                    match user {
                        Some(username) => format!(
                            "YOUR TIME HAS COME TO AN END {} :index_pointing_at_the_viewer::robot:",
                            username.mention(),
                        ),
                        None => {
                            "YOUR TIME HAS COME TO AN END :index_pointing_at_the_viewer::robot:"
                                .to_string()
                        }
                    },
                )
                .await?;

            Ok(())
        }));

    Ok(())
}

// async fn time_update(
//     ctx: Context<'_>,
//     msg_handle: ReplyHandle<'_>,
//     new_time: u32,
// ) -> Result<(), Error> {
//     msg_handle
//         .edit(
//             ctx,
//             CreateReply::default().content(format!(
//                 "{} SECOND{} REMAINING ... :bomb::robot:",
//                 new_time,
//                 if new_time == 1 { "" } else { "S" }
//             )),
//         )
//         .await?;
//
//     Ok(())
// }
