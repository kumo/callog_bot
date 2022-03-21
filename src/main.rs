use scraper::{Html, Selector};
use chrono::NaiveDateTime;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    }

  }

  Ok(())
}
