use scraper::{Html, Selector};
use chrono::NaiveDateTime;
use chrono::Utc;
use tokio::time::{sleep, Duration};
use std::fmt::{Display, Formatter};
use teloxide::{prelude2::*, utils::command::BotCommand};
use dotenv;
use std::error::Error;

#[derive(Eq, PartialEq, Debug, Clone)]
struct PhoneCall {
  who: String,
  when: NaiveDateTime
}

impl Display for PhoneCall {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let diff = Utc::now().naive_utc() - self.when;
    // println!("Phone call was {} minutes ago", diff.num_minutes());

    if diff.num_hours() > 1 {
      write!(f, "☎️ {}\n👉 {}", 
      self.who, 
      self.when.format("around %l%P on %-d %b").to_string())
    } else {
      write!(f, "☎️ {}", 
      self.who)
    } 
  }
}

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

async fn download_calls() -> Option<Vec<PhoneCall>> {
  let mut phone_calls:Vec<PhoneCall> = Vec::new();

  let resp = reqwest::get("http://192.168.1.1/callLog.lp").await.ok()?.text().await.ok()?;

  let document = Html::parse_document(&resp);
  let selector = Selector::parse(r#"table.edittable > tbody > tr"#).ok()?;

  // iterate over elements matching our selector
  // we skip the first two, because the header is in the body
  // and the second line draws the border
  for row in document.select(&selector).skip(2) {
    // grab the table cells and place into a vector
    let tds = row.text().collect::<Vec<_>>();

    // the table contains rows with no elements, due to bad spacing
    if tds.len() > 7 && tds[5] == "Ingresso" {
      // println!("{:?}", tds[1]); // phone number 
      // println!("{:?}", tds[7]); // raw date

      // TODO: I am not sure that the phone number is in UTC
      let date_time = NaiveDateTime::parse_from_str(tds[7], "%H:%M:%S - %d:%m:%Y").ok()?;
      // println!("Parsed date and time is: {}", date_time);
      // let diff = Utc::now().naive_utc() - date_time;
      // println!("Phone call was {} minutes ago", diff.num_minutes());

      let phone_call: PhoneCall = PhoneCall{who: tds[1].to_string(), when: date_time};
      phone_calls.push(phone_call);
    }
  }

  return Some(phone_calls);
}

async fn list_all_calls(bot: AutoSend<Bot>, chat_id: i64) {
  if let Some(mut phone_calls) = download_calls().await {
    if phone_calls.is_empty() {
      if let Err(_) = bot.send_message(chat_id, "There are no recent calls in memory -- was the modem recently rebooted?").await {
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
    println!("There are no phone calls in memory.");

    if let Err(_) = bot.send_message(chat_id, "Problem getting latest calls!").await {
      println!("Couldn't send list_all_calls message.");
    }
  }
}

async fn list_recent_calls(bot: AutoSend<Bot>, chat_id: i64) {
  let mut recent_phone_calls:Vec<PhoneCall> = download_calls().await
    .unwrap_or(Vec::new())
    .into_iter()
    .filter(|phone_call| Utc::now().naive_utc().signed_duration_since(phone_call.when).num_days() < 1)
    .collect();

  println!("There are {} recent phone calls.", recent_phone_calls.len());

  if recent_phone_calls.is_empty() {
    if let Err(_) = bot.send_message(chat_id, "There are no recent calls in memory -- was the modem recently rebooted?").await {
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

fn get_new_calls(last_call: &Option<PhoneCall>, phone_calls: Vec<PhoneCall>) -> Option<Vec<PhoneCall>> {
  // There are no phone calls, so there are no new calls
  if phone_calls.is_empty() {
    return None;
  }

  // There is no last phone call, so all of the calls are new calls
  if let None = last_call {
    return Some(phone_calls);
  }

  // The last call is the same as the other calls
  if phone_calls.first() == last_call.as_ref() {
    return None;
  }

  if let Some(index_element) = phone_calls.clone()
    .into_iter()
    .position(|x| &x == last_call.as_ref().unwrap()) {
      return Some(phone_calls[0..index_element].to_vec());
  } else {
    return Some(phone_calls);
  }
}

#[cfg(test)]
mod tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use super::*;

  #[test]
  fn test_no_calls() {
      assert_eq!(get_new_calls(&None, Vec::new()), None);
  }

  #[test]
  fn test_no_last_call() {
    let new_call: PhoneCall = PhoneCall{who: "new call".to_string(), when: Utc::now().naive_utc()};

    let calls: Vec<PhoneCall> = vec![new_call.clone()];

    assert_eq!(get_new_calls(&None, calls.clone()), Some(calls));
  }

  #[test]
  fn test_no_new_calls() {
    let last_call: PhoneCall = PhoneCall{who: "last call".to_string(), when: Utc::now().naive_utc()};

    assert_eq!(get_new_calls(&Some(last_call), Vec::new()), None);
  }

  #[test]
  fn test_last_call_not_found() {
    let last_call: PhoneCall = PhoneCall{who: "last call".to_string(), when: Utc::now().naive_utc()};

    let new_call_1: PhoneCall = PhoneCall{who: "new call 1".to_string(), when: Utc::now().naive_utc()};
    let new_call_2: PhoneCall = PhoneCall{who: "new call 2".to_string(), when: Utc::now().naive_utc()};
    let calls: Vec<PhoneCall> = vec![new_call_1.clone(), new_call_2.clone()];

    assert_eq!(get_new_calls(&Some(last_call), calls.clone()), Some(calls));
  }

  #[test]
  fn test_last_call_is_last_call() {
    let last_call: PhoneCall = PhoneCall{who: "last call".to_string(), when: Utc::now().naive_utc()};
    let old_call: PhoneCall = PhoneCall{who: "old call".to_string(), when: Utc::now().naive_utc()};

    let calls: Vec<PhoneCall> = vec![last_call.clone(), old_call.clone()];

    assert_eq!(get_new_calls(&Some(last_call), calls.clone()), None);
  }

  #[test]
  fn test_last_call_is_recent_call() {
    let last_call: PhoneCall = PhoneCall{who: "last call".to_string(), when: Utc::now().naive_utc()};
    let new_call_1: PhoneCall = PhoneCall{who: "new call 1".to_string(), when: Utc::now().naive_utc()};
    let new_call_2: PhoneCall = PhoneCall{who: "new call 2".to_string(), when: Utc::now().naive_utc()};
    let old_call_1: PhoneCall = PhoneCall{who: "old call 1".to_string(), when: Utc::now().naive_utc()};
    let old_call_2: PhoneCall = PhoneCall{who: "old call 2".to_string(), when: Utc::now().naive_utc()};

    let calls: Vec<PhoneCall> = vec![new_call_1.clone(), new_call_2.clone(), last_call.clone(), old_call_1.clone(), old_call_2.clone()];

    assert_eq!(get_new_calls(&Some(last_call), calls.clone()), Some(vec![new_call_1.clone(), new_call_2.clone()]));
  }

  #[test]
  fn test_last_call_is_oldest_call() {
    let last_call: PhoneCall = PhoneCall{who: "last call".to_string(), when: Utc::now().naive_utc()};
    let new_call: PhoneCall = PhoneCall{who: "new call".to_string(), when: Utc::now().naive_utc()};

    let calls: Vec<PhoneCall> = vec![new_call.clone(), last_call.clone()];

    assert_eq!(get_new_calls(&Some(last_call.clone()), calls), Some(vec![new_call.clone()]));
  }
}

async fn monitor_calls(bot: AutoSend<Bot>, chat_id: i64) {
  println!("Starting - monitor_calls");

  let mut last_call:Option<PhoneCall> = None;

  loop {
    println!("Checking calls");

    let latest_calls = download_calls()
        .await
        .and_then(|calls| get_new_calls(&last_call, calls));

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
    bot.send_message(message.chat.id, "I shouldn't speak to strangers.").await?;
  }

  match command {
      Command::Help => {
        if let Err(_) = bot.send_message(chat_id, Command::descriptions()).await {
          println!("Couldn't send answer message.");
        }
      },
      Command::Today => {
        list_recent_calls(bot.clone(), chat_id).await;
      },
      Command::Recent => {
        list_recent_calls(bot.clone(), chat_id).await;
      },
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
