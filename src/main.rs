pub mod config;

use std::collections::HashMap;
use std::sync::Arc;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::prelude::{Ready, ChannelId};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, StandardFramework, CommandResult};

#[group]
#[commands(print, make_poll)]
struct General;

struct FenbotCommand;
impl TypeMapKey for FenbotCommand {
    type Value = Arc<RwLock<HashMap<String, String>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, rct: Reaction) {        
        let poll_id = {
            let data_read = ctx.data.read().await;
            let context_lock = data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone();
            let fenbot_context = context_lock.read().await;
            fenbot_context.get("pol_msg").map_or("None".to_string(), |x| x.clone())
        }.parse::<u64>().unwrap();
        if *rct.channel_id.as_u64() == poll_id {
            let _ = rct.channel_id.say(ctx, "I saw that.").await;
        }
    }

    async fn ready(&self, _: Context, _: Ready) {
        println!("fenbot is now ready!");
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(".fennec ")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token = config::load_config();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    
    {
        //client.data = Arc::<fenbot_ctx>::new_uninit();
        let mut data = client.data.write().await;
        data.insert::<FenbotCommand>(Arc::new(RwLock::new(HashMap::default())));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    println!("fenbot is now running!");
}

#[command]
async fn print(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    //msg.reply(ctx, "Pong!").await?;
    let print_type = match args.single_quoted::<String>() {
        Ok(x) => x,
        Err(_) => {
            msg.channel_id.say(ctx, "IDK what that to print for that.").await?;
            return Ok(());
        },
    };
    match print_type.as_str(){
        "poll_id"=> {
            let poll_id = {
                let data_read = ctx.data.read().await;
                let context_lock = data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone();
                let fenbot_context = context_lock.read().await;
                fenbot_context.get("pol_msg").map_or("None".to_string(), |x| x.clone())
            };
            msg.channel_id.say(ctx, format!("The poll message id is: {:?}", poll_id)).await?;
        },
        "this_channel"=> {
            msg.channel_id.say(ctx, format!("This channel's id is: {:?}", msg.channel_id.as_u64())).await?;
        },
        _=> {
            msg.channel_id.say(ctx, format!("{:?} not programmed.", print_type)).await?;
        },
    }

    Ok(())
}

#[command]
async fn make_poll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pid = args.single_quoted::<u64>().unwrap();
    println!("make_poll args: {:?}", pid);

    let ctp = ChannelId::from(pid);
    let say_good = ctp.say(ctx, "We good?").await;
    match say_good {
        Ok(_) => {
            let pol_msg = pid.to_string();

            let context_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone()
            };

            {
                let mut fctx = context_lock.write().await;
                let entry = fctx.entry("pol_msg".to_string()).or_insert("pol_msg".to_string());
                *entry = pol_msg.clone();
            }
            config::write_poll_id(pid);
            msg.channel_id.say(ctx, "Poll generated").await?
            
        },
        Err(e) => msg.channel_id.say(ctx, format!("Could not generate poll, error: {:?}", e)).await?,
    };
    Ok(())
}