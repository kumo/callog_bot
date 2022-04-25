use chrono::{NaiveDateTime, Utc};
use std::fmt::{Display, Formatter};
use visdom::Vis;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct PhoneCall {
    pub who: String,
    pub when: NaiveDateTime,
}

impl PhoneCall {
    pub fn is_today(&self) -> bool {
        Utc::now()
            .naive_utc()
            .signed_duration_since(self.when)
            .num_days()
            == 0
    }
}

impl PhoneCall {
    pub fn is_recent(&self) -> bool {
        Utc::now()
            .naive_utc()
            .signed_duration_since(self.when)
            .num_minutes()
            <= 20
    }
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

impl TryFrom<&[String]> for PhoneCall {
    type Error = ();

    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        let who = value[0].to_string();
        if let Ok(when) = NaiveDateTime::parse_from_str(&value[3], "%H:%M:%S - %d:%m:%Y") {
            Ok(PhoneCall { who, when })
        } else {
            warn!("Couldn't parse date {}", &value[3]);
            Err(())
        }
    }
}

pub async fn download_calls() -> Option<Vec<PhoneCall>> {
    let resp = reqwest::get("http://192.168.1.1/callLog.lp")
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let tds = Vis::load(resp)
        .ok()?
        .find("table.edittable > tr > td.fontSize");

    let phone_calls = tds
        .map(|_index, ele| String::from(Vis::dom(ele).text()))
        .chunks_exact(5)
        .filter(|data| data[2] == "Ingresso")
        .filter_map(|data| PhoneCall::try_from(data).ok())
        .collect();

    return Some(phone_calls);
}

pub fn get_new_calls(
    last_call: &Option<PhoneCall>,
    phone_calls: Vec<PhoneCall>,
) -> Option<Vec<PhoneCall>> {
    // There are no phone calls, so there are no new calls
    if phone_calls.is_empty() {
        debug!("No phone calls");
        return None;
    }

    // There is no last phone call, so all of the calls are new calls,
    // but only return today's calls
    if let None = last_call {
        debug!("No last call, returning today's calls.");
        // TODO return today's calls
        let today_calls: Vec<PhoneCall> = phone_calls
            .into_iter()
            .filter(|phone_call| phone_call.is_recent())
            .collect();

        return Some(today_calls);
    }

    // The last call is the same as the other calls
    if phone_calls.first() == last_call.as_ref() {
        debug!("Latest phone call is same, returning none.");
        return None;
    }

    if let Some(index_element) = phone_calls
        .clone()
        .into_iter()
        .position(|x| &x == last_call.as_ref().unwrap())
    {
        debug!(
            "Last phone call found, returning some calls 0..{}",
            index_element
        );
        return Some(phone_calls[0..index_element].to_vec());
    } else {
        debug!("Last phone call not found, returning all calls.");
        return Some(phone_calls);
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

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
    fn test_no_last_return_recent_calls() {
        let new_call: PhoneCall = PhoneCall {
            who: "new call".to_string(),
            when: Utc::now().naive_utc(),
        };
        let old_call: PhoneCall = PhoneCall {
            who: "old call".to_string(),
            when: Utc::now()
                .checked_sub_signed(Duration::seconds(60 * 31))
                .unwrap()
                .naive_utc(),
        };

        let calls: Vec<PhoneCall> = vec![new_call.clone(), old_call.clone()];

        assert_eq!(
            get_new_calls(&None, calls.clone()),
            Some(vec![new_call.clone()])
        );
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
