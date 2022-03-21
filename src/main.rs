use scraper::{Html, Selector};
use chrono::NaiveDateTime;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let resp = reqwest::get("http://192.168.1.1/callLog.lp").await?.text().await?;

  let document = Html::parse_document(&resp);
  let selector = Selector::parse(r#"table.edittable > tbody > tr"#).unwrap();

  // iterate over elements matching our selector
  for row in document.select(&selector).skip(2) {
    // grab the headline text and place into a vector
    let tds = row.text().collect::<Vec<_>>();

    // let output_tds = tds.drain_filter(|td| td.len() > 1).collect::<Vec<_>>();
    if tds.len() > 7 && tds[5] == "Ingresso" {
      println!("{:?}", tds[1]);
      println!("{:?}", tds[7]);
      let date_time = NaiveDateTime::parse_from_str(tds[7], "%H:%M:%S - %d:%m:%Y")?;
      println!("Parsed date and time is: {}", date_time);
      let diff = Utc::now().naive_utc() - date_time;
      println!("Total time taken to run is {}", diff.num_minutes());
    }

  }

  Ok(())
}
