use chrono::{NaiveDateTime, Utc};
use std::fmt::{Display, Formatter};
use visdom::Vis;

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

impl TryFrom<&[String]> for PhoneCall {
    type Error = ();

    fn try_from(value: &[String]) -> Result<Self, Self::Error> {
        let who = value[0].to_string();
        if let Ok(when) = NaiveDateTime::parse_from_str(&value[3], "%H:%M:%S - %d:%m:%Y") {
            Ok(PhoneCall { who, when })
        } else {
            Err(())
        }
    }
}

pub struct LineStats {
    pub upload: u32,
    pub download: u32,
}

impl Display for LineStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ratio = self.download / self.upload;

        if ratio < 1 {
            write!(
                f,
                "⚠️ Download speed is lower than upload speed, please reboot!"
            )
        } else if ratio < 2 {
            write!(
                f,
                "⚠️ Download speed is similar to upload speed, maybe reboot!"
            )
        } else {
            write!(f, "Internet connect seems fine.")
        }
    }
}

fn parse_int(input: &str) -> Option<u32> {
    input
        .chars()
        .skip_while(|ch| !ch.is_digit(10))
        .take_while(|ch| ch.is_digit(10))
        .fold(None, |acc, ch| {
            ch.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
        })
}

impl TryFrom<Vec<String>> for LineStats {
    type Error = ();

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let download = parse_int(&value[1]);
        let upload = parse_int(&value[2]);

        if let (Some(download), Some(upload)) = (download, upload) {
            Ok(LineStats { download, upload })
        } else {
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

pub async fn download_stats() -> Option<LineStats> {
    let home_resp = reqwest::get("http://192.168.1.1/home.lp")
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let tds = Vis::load(home_resp)
        .ok()?
        .find("table.tablecontainttbl > tr > td.fcolor");
    println!("There are {} matching cells.", tds.length());

    let texts = tds.map(|_index, ele| String::from(Vis::dom(ele).text()));

    // check the external IP and download/upload speeds
    println!("IP: {}", texts[0]);
    println!("Download: {}", texts[1]);
    println!("Upload: {}", texts[2]);

    return LineStats::try_from(texts).ok();

    // if texts[1].len() > texts[2].len() {
    //     println!("Download speed is faster than upload speed.");
    // } else if texts[1].len() < texts[2].len() {
    //     println!("Download speed is slower than upload speed!");
    // } else {
    //     if texts[1] > texts[2] {
    //         println!("Download speed is faster but also similar to upload speed.");
    //     } else {
    //         println!("Download speed is slower but also similar to upload speed.");
    //     }
    // }
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
