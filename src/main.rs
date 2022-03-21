use scraper::{Html, Selector};
use chrono::NaiveDateTime;
use chrono::Utc;

#[derive(Eq, PartialEq)]
#[derive(Debug)]
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
      println!("{:?}", tds[1]); // phone number 
      println!("{:?}", tds[7]); // raw date

      // TODO: I am not sure that the phone number is in UTC
      let date_time = NaiveDateTime::parse_from_str(tds[7], "%H:%M:%S - %d:%m:%Y")?;
      println!("Parsed date and time is: {}", date_time);
      let diff = Utc::now().naive_utc() - date_time;
      println!("Phone call was {} minutes ago", diff.num_minutes());

      let phone_call: PhoneCall = PhoneCall{who: tds[1].to_string(), when: date_time};
      phone_calls.push(phone_call);
    }
  }

  return Ok(phone_calls);
}

async fn list_all_calls() -> Result<(), Box<dyn std::error::Error>> {
  let phone_calls:Vec<PhoneCall> = get_calls().await?;

  if phone_calls.is_empty() {
    println!("There are no phone calls in memory.")
  } else {
    println!("There are {} phone calls.", phone_calls.len());
  }

  Ok(())
}

async fn list_recent_calls() -> Result<(), Box<dyn std::error::Error>> {
  let recent_phone_calls:Vec<PhoneCall> = get_calls().await?
    .into_iter()
    .filter(|phone_call| Utc::now().naive_utc().signed_duration_since(phone_call.when).num_days() < 1)
    .collect();

  println!("There are {} recent phone calls.", recent_phone_calls.len());

  Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  list_all_calls().await?;

  Ok(())
}
