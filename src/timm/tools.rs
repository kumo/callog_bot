use std::collections::HashMap;

pub async fn reboot() -> Option<reqwest::Response> {
    let tool_resp = reqwest::get("http://192.168.1.1/tool.lp").await.ok()?;
    let mut cookies = tool_resp.cookies();

    let client = reqwest::Client::new();

    if let Some(cookie) = cookies.next() {
        let mut params = HashMap::new();
        params.insert("action", "saveRestart");
        params.insert("rn", cookie.value());

        let post_res = client
            .post("http://192.168.1.1/resetAG.lp")
            .form(&params)
            .send()
            .await;

        debug!("Reboot response: {:?}", post_res);

        return post_res.ok();
    }

    None
}
