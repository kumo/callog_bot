use std::fmt::{Display, Formatter};
use visdom::Vis;

#[derive(PartialEq)]
pub enum LineSpeed {
    Bad,
    Slow,
    Normal,
}

pub struct LineStats {
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
        .skip_while(|ch| !ch.is_digit(10))
        .take_while(|ch| ch.is_digit(10))
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
        let download = parse_int(&value[1]);
        let upload = parse_int(&value[2]);

        if let (Some(download), Some(upload)) = (download, upload) {
            debug!("Creating now stats: {}, {}", download, upload);
            let ratio = download / upload;

            Ok(LineStats {
                download,
                upload,
                speed: LineSpeed::from(ratio),
            })
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

    let texts = tds.map(|_index, ele| String::from(Vis::dom(ele).text()));

    // check the external IP and download/upload speeds
    debug!("IP: {}", texts[0]);
    debug!("Download: {}", texts[1]);
    debug!("Upload: {}", texts[2]);

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
