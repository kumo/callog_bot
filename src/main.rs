use scraper::{Html, Selector};
use chrono::NaiveDateTime;
use chrono::Utc;
use tokio::time::{sleep, Duration};

#[derive(Eq, PartialEq, Debug, Clone)]
struct PhoneCall {
  who: String,
  when: NaiveDateTime
}

async fn get_calls() -> Result<Vec<PhoneCall>, Box<dyn std::error::Error>> {
  let mut phone_calls:Vec<PhoneCall> = Vec::new();

  let resp = reqwest::get("http://192.168.1.1/callLog.lp").await?.text().await?;

  let document = Html::parse_document(&resp);
  let selector = Selector::parse(r#"table.edittable > tbody > tr"#).unwrap();

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
      let date_time = NaiveDateTime::parse_from_str(tds[7], "%H:%M:%S - %d:%m:%Y").unwrap();
      // println!("Parsed date and time is: {}", date_time);
      // let diff = Utc::now().naive_utc() - date_time;
      // println!("Phone call was {} minutes ago", diff.num_minutes());

      let phone_call: PhoneCall = PhoneCall{who: tds[1].to_string(), when: date_time};
      phone_calls.push(phone_call);
    }
  }

  return Ok(phone_calls);
}

async fn list_all_calls() {
  let phone_calls:Vec<PhoneCall> = get_calls().await.unwrap();

  if phone_calls.is_empty() {
    println!("There are no phone calls in memory.")
  } else {
    println!("There are {} phone calls.", phone_calls.len());
  }
}

async fn list_recent_calls() {
  let recent_phone_calls:Vec<PhoneCall> = get_calls().await.unwrap()
    .into_iter()
    .filter(|phone_call| Utc::now().naive_utc().signed_duration_since(phone_call.when).num_days() < 1)
    .collect();

  println!("There are {} recent phone calls.", recent_phone_calls.len());
}

async fn monitor_calls() {
  println!("Starting - monitor_calls");

  let mut last_call:Option<PhoneCall> = None;

  loop {
      sleep(Duration::from_secs(6)).await;

      println!("Checking calls");
      let phone_calls:Vec<PhoneCall> = get_calls().await.unwrap();

      if last_call == None {
        println!("Print all calls");
        for phone_call in &phone_calls {
          println!("{:?}", phone_call);
        }

        if let Some(call) = Some(phone_calls.last().cloned()) {
          last_call = call;
        }
      } else {
        if let Some(index_element) = phone_calls
          .iter()
          .position(|x| x == last_call.as_mut().unwrap()) {
            println!("{:?}", index_element);

            if index_element == phone_calls.len() - 1 {
              println!("No new calls.");
            } else {
              println!("Print one or more recent call(s)");
              for phone_call in &phone_calls[index_element..] {
                println!("{:?}", phone_call);
              }
    
              if let Some(call) = Some(phone_calls.last().cloned()) {
                last_call = call;
              }
            }
        } else {
          println!("Print all calls (last call not found)");
          for phone_call in &phone_calls {
            println!("{:?}", phone_call);
          }
  
          if let Some(call) = Some(phone_calls.last().cloned()) {
            last_call = call;
          }
        }
      }
  }
  // println!("HI")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let tasks = vec![
    tokio::spawn(async move { monitor_calls().await }),
    tokio::spawn(async move { list_all_calls().await }),
  ];

  futures::future::join_all(tasks).await;

  Ok(())
}
