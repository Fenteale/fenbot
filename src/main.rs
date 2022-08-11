pub mod config;

use std::sync::Arc;

use config::Config;
use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::model::channel::Reaction;
use serenity::model::prelude::{Ready, ChannelId, RoleId};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, StandardFramework, CommandResult};

#[group]
#[commands(print, make_poll)]
struct General;

struct FenbotCommand;
impl TypeMapKey for FenbotCommand {
    type Value = Arc<RwLock<Config>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, rct: Reaction) {     
        //println!("Hey.  This is a test {:?}", rct.emoji.unicode_eq("🦊"));   
        let c = {
            let data_read = ctx.data.read().await;
            let context_lock = data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone();
            let fc = context_lock.read().await;
            fc.clone()
        };

        if *rct.message_id.as_u64() == c.poll_id {
            let g = rct.guild_id.unwrap();
            //let rta = RoleId::from(c.roles.fox);
            for r in c.roles {
                if rct.emoji.unicode_eq(r.emoji.as_str()) {
                    let rta = RoleId::from(r.id);
                    let _ = g.member(ctx.clone(), rct.user_id.unwrap())
                        .await.unwrap()
                        .add_role(ctx.clone(), rta)
                        .await.expect("Could not add role to user.");
                    println!("Added role {:?} to {:?}.", rta.to_role_cached(ctx.clone()).unwrap().name, rct.user(ctx.clone()).await.unwrap().name);
                }
            }
            
        }
    }

    async fn reaction_remove(&self, ctx: Context, rct: Reaction) {
        let c = {
            let data_read = ctx.data.read().await;
            let context_lock = data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone();
            let fc = context_lock.read().await;
            fc.clone()
        };

        if *rct.message_id.as_u64() == c.poll_id {
            let g = rct.guild_id.unwrap();
            //let rta = RoleId::from(c.roles.fox);
            for r in c.roles {
                if rct.emoji.unicode_eq(r.emoji.as_str()) {
                    let rta = RoleId::from(r.id);
                    let _ = g.member(ctx.clone(), rct.user_id.unwrap())
                        .await.unwrap()
                        .remove_role(ctx.clone(), rta)
                        .await.expect("Could not add role to user.");
                        println!("Removed role {:?} from {:?}.", rta.to_role_cached(ctx.clone()).unwrap().name, rct.user(ctx.clone()).await.unwrap().name);
                }
            }
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
    let c = config::load_config();
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(c.token.clone(), intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");
    
    {
        let mut data = client.data.write().await;
        data.insert::<FenbotCommand>(Arc::new(RwLock::new(c.clone())));
        
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
    println!("fenbot is now running!");
}

#[command]
async fn print(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
                let fc = context_lock.read().await;
                fc.poll_id.clone()
            };
            msg.channel_id.say(ctx, format!("The poll message id is: {:?}", poll_id)).await?;
        },
        "this_channel"=> {
            msg.channel_id.say(ctx, format!("This channel's id is: {:?}", msg.channel_id.as_u64())).await?;
        },
        "source"=> {
            msg.channel_id.say(ctx, format!("The source code for this bot is hosted at: https://github.com/Fenteale/fenbot")).await?;
        },
        _=> {
            msg.channel_id.say(ctx, format!("{:?} not programmed.", print_type)).await?;
        },
    }

    Ok(())
}

#[command]
async fn make_poll(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let c = {
        let data_read = ctx.data.read().await;
        let context_lock = data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone();
        let fc = context_lock.read().await;
        fc.clone()
    };
    if *msg.author.id.as_u64() != c.admin {
        msg.channel_id.say(ctx, "You are not a fennec fox.").await?;
        return Ok(());
    }
    let pid = args.single_quoted::<u64>().unwrap();
    println!("make_poll args: {:?}", pid);

    let ctp = ChannelId::from(pid);
    let say_good = ctp.say(ctx, "Poll Message").await;
    match say_good {
        Ok(nmsg) => {
            let pol_msg = nmsg.id.as_u64();

            let context_lock = {
                let data_read = ctx.data.read().await;
                data_read.get::<FenbotCommand>().expect("FenbotCommand in TypeMap.").clone()
            };

            {
                let mut fctx = context_lock.write().await;
                fctx.poll_id = pol_msg.clone();
            }
            config::write_poll_id(pol_msg.clone());
            msg.channel_id.say(ctx, "Poll generated").await?
            
        },
        Err(e) => msg.channel_id.say(ctx, format!("Could not generate poll, error: {:?}", e)).await?,
    };
    Ok(())
}