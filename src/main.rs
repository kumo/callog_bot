use chrono::Utc;
use dotenv;
use std::error::Error;
use teloxide::{prelude2::*, utils::command::BotCommand};
use tokio::time::{sleep, Duration};

mod timm;
use timm::PhoneCall;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "display today's calls.")]
    Today,
    #[command(description = "display recent calls.")]
    Recent,
    #[command(description = "display all calls.")]
    All,
}

async fn list_all_calls(bot: AutoSend<Bot>, chat_id: i64) {
    if let Some(mut phone_calls) = timm::download_calls().await {
        if phone_calls.is_empty() {
            if let Err(_) = bot
                .send_message(
                    chat_id,
                    "There are no recent calls in memory -- was the modem recently rebooted?",
                )
                .await
            {
                println!("Couldn't send list_all_calls message.");
            }
        } else {
            println!("There are new calls");

            phone_calls.reverse();
            for phone_call in &phone_calls {
                println!("{}", phone_call);

                if let Err(_) = bot.send_message(chat_id, format!("{}", phone_call)).await {
                    println!("Couldn't send list_all_calls message.");
                }
            }

            println!("There are {} phone calls.", phone_calls.len());
        }
    } else {
        println!("There might be no phone calls in memory.");

        if let Err(_) = bot
            .send_message(chat_id, "Problem getting latest calls!")
            .await
        {
            println!("Couldn't send list_all_calls message.");
        }
    }
}

async fn list_recent_calls(bot: AutoSend<Bot>, chat_id: i64) {
    let mut recent_phone_calls: Vec<PhoneCall> = timm::download_calls()
        .await
        .unwrap_or(Vec::new())
        .into_iter()
        .filter(|phone_call| {
            Utc::now()
                .naive_utc()
                .signed_duration_since(phone_call.when)
                .num_days()
                < 1
        })
        .collect();

    println!("There are {} recent phone calls.", recent_phone_calls.len());

    if recent_phone_calls.is_empty() {
        if let Err(_) = bot
            .send_message(
                chat_id,
                "There are no recent calls in memory -- was the modem recently rebooted?",
            )
            .await
        {
            println!("Couldn't send list_recent_calls message.");
        }
    } else {
        recent_phone_calls.reverse();
        for phone_call in &recent_phone_calls {
            println!("{}", phone_call);

            if let Err(_) = bot.send_message(chat_id, format!("{}", phone_call)).await {
                println!("Couldn't send list_recent_calls message.");
            }
        }
    }
}

async fn monitor_calls(bot: AutoSend<Bot>, chat_id: i64) {
    println!("Starting - monitor_calls");

    let mut last_call: Option<PhoneCall> = None;

    loop {
        println!("Checking calls");

        let latest_calls = timm::download_calls()
            .await
            .and_then(|calls| timm::get_new_calls(&last_call, calls));

        if let Some(mut latest_calls) = latest_calls {
            println!("There are new calls");

            latest_calls.reverse();
            for phone_call in &latest_calls {
                println!("{}", phone_call);

                if let Err(_) = bot.send_message(chat_id, format!("{}", phone_call)).await {
                    println!("Couldn't send monitor_calls message.");
                }
            }

            if let Some(call) = Some(latest_calls.last().cloned()) {
                last_call = call;
            }
        } else {
            println!("No calls found.")
        }

        sleep(Duration::from_secs(60)).await;
    }
}

async fn answer(
    bot: AutoSend<Bot>,
    message: Message,
    command: Command,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let chat_id: i64 = envmnt::get_parse("CHAT_ID").unwrap();

    if message.chat.id != chat_id {
        bot.send_message(message.chat.id, "I shouldn't speak to strangers.")
            .await?;
        println!("I shouldn't talk to strangers: {}", message.chat.id);
    }

    match command {
        Command::Help => {
            if let Err(_) = bot.send_message(chat_id, Command::descriptions()).await {
                println!("Couldn't send answer message.");
            }
        }
        Command::Today => {
            list_recent_calls(bot.clone(), chat_id).await;
        }
        Command::Recent => {
            list_recent_calls(bot.clone(), chat_id).await;
        }
        Command::All => {
            list_all_calls(bot.clone(), chat_id).await;
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let chat_id: i64 = envmnt::get_parse("CHAT_ID").unwrap();

    let bot = Bot::from_env().auto_send();
    let handler = teloxide::repls2::commands_repl(bot.clone(), answer, Command::ty());

    let tasks = vec![
        tokio::spawn(async move { monitor_calls(bot.clone(), chat_id).await }),
        tokio::spawn(async move { handler.await }),
    ];

    futures::future::join_all(tasks).await;

    Ok(())
}
