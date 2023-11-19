use std::fmt::{Display, Formatter};
use visdom::Vis;

#[derive(PartialEq, Eq, Debug)]
pub enum LineSpeed {
    Bad,
    Slow,
    Normal,
}

#[derive(PartialEq, Eq, Debug)]
pub struct LineStats {
    pub ip: String,
    pub upload: u32,
    pub download: u32,
    pub speed: LineSpeed,
}

impl Display for LineStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}🔻 {}kbps\n🔺 {}kbps",
            self.speed, self.download, self.upload
        )
    }
}

impl Display for LineSpeed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LineSpeed::Bad => "⚠️ Download speed is lower than upload speed, please reboot!\n",
                LineSpeed::Slow => "⚠️ Download speed is similar to upload speed, maybe reboot!\n",
                LineSpeed::Normal => "Download speed seems normal.\n",
            }
        )
    }
}

fn parse_int(input: &str) -> Option<u32> {
    input
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit())
        .fold(None, |acc, ch| {
            ch.to_digit(10).map(|b| acc.unwrap_or(0) * 10 + b)
        })
}

impl From<u32> for LineSpeed {
    fn from(value: u32) -> Self {
        if value < 1 {
            LineSpeed::Bad
        } else if value < 2 {
            LineSpeed::Slow
        } else {
            LineSpeed::Normal
        }
    }
}

impl TryFrom<Vec<String>> for LineStats {
    type Error = ();

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err(());
        }

        let download = parse_int(&value[1]);
        let upload = parse_int(&value[2]);

        if let (Some(download), Some(upload)) = (download, upload) {
            debug!("Creating now stats: {}, {}", download, upload);
            if upload < 1 {
                Err(())
            } else {
                let ratio = download / upload;

                let ip = value[0].to_string();

                Ok(LineStats {
                    ip,
                    download,
                    upload,
                    speed: LineSpeed::from(ratio),
                })
            }
        } else {
            warn!(
                "Couldn't parse download {} or upload {}",
                &value[1], &value[2]
            );
            Err(())
        }
    }
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
    debug!("There are {} matching cells.", tds.length());

    let texts = tds.map(|_index, ele| Vis::dom(ele).text());

    // check the external IP and download/upload speeds
    debug!("IP: {}", texts[0]);
    debug!("Download: {}", texts[1]);
    debug!("Upload: {}", texts[2]);

    LineStats::try_from(texts).ok()

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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_no_stats() {
        let stats = LineStats::try_from(Vec::new());

        assert_eq!(stats.is_err(), true);
    }

    #[test]
    fn test_no_number_stats() {
        let stats = LineStats::try_from(vec![
            "one".to_string(),
            "two".to_string(),
            "three".to_string(),
        ]);

        assert_eq!(stats.is_err(), true);
    }

    #[test]
    fn test_equal_number_stats() {
        let stats = LineStats::try_from(vec!["1".to_string(), "1".to_string(), "1".to_string()]);

        assert_eq!(stats.is_ok(), true);
        assert_eq!(
            stats,
            Ok(LineStats {
                ip: String::from("1"),
                upload: 1,
                download: 1,
                speed: LineSpeed::Slow
            })
        );
    }

    #[test]
    fn test_zero_upload_stats() {
        let stats = LineStats::try_from(vec!["0".to_string(), "0".to_string(), "0".to_string()]);

        assert_eq!(stats.is_ok(), false);
        assert_eq!(stats, Err(()));
    }

    #[test]
    fn test_equal_stats() {
        let stats = LineStats::try_from(vec!["5".to_string(), "5".to_string(), "5".to_string()]);

        assert_eq!(stats.is_ok(), true);
        assert_eq!(
            stats,
            Ok(LineStats {
                ip: String::from("5"),
                upload: 5,
                download: 5,
                speed: LineSpeed::Slow
            })
        );
    }

    #[test]
    fn test_normal_stats() {
        let stats = LineStats::try_from(vec![
            "1.2.3.4".to_string(),
            "12945kbps".to_string(),
            "3143kbps".to_string(),
        ]);

        assert_eq!(stats.is_ok(), true);
        assert_eq!(
            stats,
            Ok(LineStats {
                ip: String::from("1.2.3.4"),
                upload: 3143,
                download: 12945,
                speed: LineSpeed::Normal
            })
        );
    }
}
