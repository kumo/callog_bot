use chrono::{NaiveDateTime, Utc};
use scraper::{Html, Selector};
use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PhoneCall {
    pub who: String,
    pub when: NaiveDateTime,
}

impl Display for PhoneCall {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let diff = Utc::now().naive_utc() - self.when;
        // println!("Phone call was {} minutes ago", diff.num_minutes());

        if diff.num_hours() > 1 {
            write!(
                f,
                "☎️ {}\n👉 {}",
                self.who,
                self.when.format("around %l%P on %-d %b").to_string()
            )
        } else {
            write!(f, "☎️ {}", self.who)
        }
    }
}

pub async fn download_calls() -> Option<Vec<PhoneCall>> {
    let mut phone_calls: Vec<PhoneCall> = Vec::new();

    let resp = reqwest::get("http://192.168.1.1/callLog.lp")
        .await
        .ok()?
        .text()
        .await
        .ok()?;

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

            let phone_call: PhoneCall = PhoneCall {
                who: tds[1].to_string(),
                when: date_time,
            };
            phone_calls.push(phone_call);
        }
    }

    return Some(phone_calls);
}

pub fn get_new_calls(
    last_call: &Option<PhoneCall>,
    phone_calls: Vec<PhoneCall>,
) -> Option<Vec<PhoneCall>> {
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

    if let Some(index_element) = phone_calls
        .clone()
        .into_iter()
        .position(|x| &x == last_call.as_ref().unwrap())
    {
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
        let new_call: PhoneCall = PhoneCall {
            who: "new call".to_string(),
            when: Utc::now().naive_utc(),
        };

        let calls: Vec<PhoneCall> = vec![new_call.clone()];

        assert_eq!(get_new_calls(&None, calls.clone()), Some(calls));
    }

    #[test]
    fn test_no_new_calls() {
        let last_call: PhoneCall = PhoneCall {
            who: "last call".to_string(),
            when: Utc::now().naive_utc(),
        };

        assert_eq!(get_new_calls(&Some(last_call), Vec::new()), None);
    }

    #[test]
    fn test_last_call_not_found() {
        let last_call: PhoneCall = PhoneCall {
            who: "last call".to_string(),
            when: Utc::now().naive_utc(),
        };

        let new_call_1: PhoneCall = PhoneCall {
            who: "new call 1".to_string(),
            when: Utc::now().naive_utc(),
        };
        let new_call_2: PhoneCall = PhoneCall {
            who: "new call 2".to_string(),
            when: Utc::now().naive_utc(),
        };
        let calls: Vec<PhoneCall> = vec![new_call_1.clone(), new_call_2.clone()];

        assert_eq!(get_new_calls(&Some(last_call), calls.clone()), Some(calls));
    }

    #[test]
    fn test_last_call_is_last_call() {
        let last_call: PhoneCall = PhoneCall {
            who: "last call".to_string(),
            when: Utc::now().naive_utc(),
        };
        let old_call: PhoneCall = PhoneCall {
            who: "old call".to_string(),
            when: Utc::now().naive_utc(),
        };

        let calls: Vec<PhoneCall> = vec![last_call.clone(), old_call.clone()];

        assert_eq!(get_new_calls(&Some(last_call), calls.clone()), None);
    }

    #[test]
    fn test_last_call_is_recent_call() {
        let last_call: PhoneCall = PhoneCall {
            who: "last call".to_string(),
            when: Utc::now().naive_utc(),
        };
        let new_call_1: PhoneCall = PhoneCall {
            who: "new call 1".to_string(),
            when: Utc::now().naive_utc(),
        };
        let new_call_2: PhoneCall = PhoneCall {
            who: "new call 2".to_string(),
            when: Utc::now().naive_utc(),
        };
        let old_call_1: PhoneCall = PhoneCall {
            who: "old call 1".to_string(),
            when: Utc::now().naive_utc(),
        };
        let old_call_2: PhoneCall = PhoneCall {
            who: "old call 2".to_string(),
            when: Utc::now().naive_utc(),
        };

        let calls: Vec<PhoneCall> = vec![
            new_call_1.clone(),
            new_call_2.clone(),
            last_call.clone(),
            old_call_1.clone(),
            old_call_2.clone(),
        ];

        assert_eq!(
            get_new_calls(&Some(last_call), calls.clone()),
            Some(vec![new_call_1.clone(), new_call_2.clone()])
        );
    }

    #[test]
    fn test_last_call_is_oldest_call() {
        let last_call: PhoneCall = PhoneCall {
            who: "last call".to_string(),
            when: Utc::now().naive_utc(),
        };
        let new_call: PhoneCall = PhoneCall {
            who: "new call".to_string(),
            when: Utc::now().naive_utc(),
        };

        let calls: Vec<PhoneCall> = vec![new_call.clone(), last_call.clone()];

        assert_eq!(
            get_new_calls(&Some(last_call.clone()), calls),
            Some(vec![new_call.clone()])
        );
    }
}
