use anyhow::Result;
use bytes::Bytes;
use flate2::bufread::ZlibDecoder;
use reqwest::*;
use std::io::Cursor;
use std::{io::Read, str};

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
        // Skip flush
        let tmp = if &head_line[0..4] == "0000" {
            &head_line[4..]
        } else {
            head_line
        };
        // Skip length encoding and take the 40 byte hex-encoded digest
        let digest = &tmp[4..44];
        Ok(digest.to_string())
    }

    pub async fn fetch_pack(&self, commit: &str) -> Result<()> {
        let req = self
            .client
            .post(format!("{}/git-upload-pack", self.url))
            .header("Content-Type", "application/x-git-upload-pack-request")
            .body(format!("0032want {}\n00000009done\n", commit))
            .build()?;
        let res = self.client.execute(req).await?;
        let body = res.bytes().await?;
        //We're doing a clone so there is no common ancestor and hence we will always get NAK
        assert!(str::from_utf8(&body[..8]) == Ok("0008NAK\n"));
        ApiClient::parse_pack(&body[8..])
    }

    // https://github.com/git/git/blob/795ea8776befc95ea2becd8020c7a284677b4161/Documentation/gitformat-pack.txt
    fn parse_pack(pack: &[u8]) -> Result<()> {
        // Header
        assert!(str::from_utf8(&pack[..4]) == Ok("PACK"));
        // Version number
        assert!(u32::from_be_bytes(pack[4..8].try_into().unwrap()) == 2);

        let num_objects = u32::from_be_bytes(pack[8..12].try_into().unwrap());
        dbg!(num_objects);
        let mut pack = &pack[12..];
        let mut count = 0;
        while count < num_objects {
            // TODO handle refs delta at least
            let tpe = (pack[0] & 0x70) >> 4;
            dbg!(tpe);
            assert!(tpe <= 4);
            let mut len: usize = (pack[0] & 0x0f).into();
            dbg!(len);
            let mut idx = 1;
            while (pack[idx - 1] & (1 << 7)) != 0 {
                let tmp: usize = u8::from_be(pack[idx] & 0x7f).into();
                let tmp = tmp << (4 + 7 * (idx - 1));
                len |= tmp;
                idx += 1;
            }
            dbg!(len);
            let mut z = ZlibDecoder::new(Cursor::new(&pack[idx..]));
            let mut buf = Vec::with_capacity(len);
            z.read_to_end(&mut buf)?;
            assert!(z.total_out() as usize == len);
            count += 1;
            pack = &pack[idx + (z.total_in() as usize)..]
        }
        Ok(())
    }
}
