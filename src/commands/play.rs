/*
Copyright © 2022 Kyagara
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at
    http://www.apache.org/licenses/LICENSE-2.0
Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

use serenity::framework::standard::macros::command;
use serenity::framework::standard::Args;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

use tracing::{error, info};

use crate::check_msg;

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.reply(&ctx.http, "URL not provided.").await);

            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(msg.reply(&ctx.http, "The URL provided is not valid.").await);

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "You are not in a voice channel").await);

            return Ok(());
        }
    };

    let manager = songbird::get(ctx).await.unwrap();

    let _handler = manager.join(guild.id, connect_to).await;

    if let Some(handler) = manager.get(guild.id) {
        // This mut handler doesn't look good, it needs to be mut for the handler.enqueue_source() below
        // Can this be changed?
        let mut handler = handler.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                error!("error starting source: {:?}", why);

                check_msg(msg.reply(&ctx.http, "Error starting source.").await);

                return Ok(());
            }
        };

        info!("adding track '{}' to queue", url);

        let metadata = source.metadata.clone();

        handler.enqueue_source(source);

        let queue_length = handler.queue().len();

        drop(handler);

        check_msg(
            msg.channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.colour(0xfcd34d)
                            .thumbnail(metadata.thumbnail.unwrap())
                            .title(format!("Playing: {}", metadata.title.unwrap()))
                            .url(url)
                            .field("Tracks in queue:", queue_length, false)
                            .footer(|f| {
                                f.icon_url(msg.author.avatar_url().unwrap());
                                f.text(msg.author.name.clone());
                                f
                            })
                    })
                })
                .await,
        );

        return Ok(());
    }

    check_msg(
        msg.reply(&ctx.http, "Not in a voice channel to play in.")
            .await,
    );

    Ok(())
}
