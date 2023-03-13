use anyhow::Result;
use reqwest::*;

pub struct ApiClient<'a> {
    client: Client,
    url: &'a str,
}

// https://git-scm.com/book/en/v2/Git-Internals-Transfer-Protocols
// https://git-scm.com/docs/protocol-common
// https://git-scm.com/docs/pack-protocol/en
impl<'a> ApiClient<'a> {
    pub fn new(url: &'a str) -> ApiClient<'a> {
        ApiClient {
            client: Client::new(),
            url,
        }
    }

    pub async fn head(&self) -> Result<String> {
        let req = self
            .client
            .get(format!("{}/info/refs?service=git-upload-pack", self.url))
            .build()?;

        let res = self.client.execute(req).await?;
        let body = res.text().await?;
        let head_line = body.lines().find(|l| l.contains("HEAD")).unwrap();
        let tmp = if &head_line[0..4] == "0000" {
            &head_line[4..]
        } else {
            head_line
        };
        let digest = &tmp[4..44];
        Ok(digest.to_string())
    }

    pub async fn fetch_pack(&self, commit: &str) -> Result<()> {
        let req = self
            .client
            .post(format!("{}/git-upload-pack", self.url))
            .header("Content-Type", "x-git-upload-request")
            .body(format!("0032want {}\n00000009done\n", commit))
            .build()?;

        let res = self.client.execute(req).await?;
        dbg!(res);
        Ok(())
    }
}
