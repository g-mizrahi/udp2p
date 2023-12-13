pub mod discovery {
    //! This module contains functions to interact with the central REST server.
    //! Its goal is to provide all utilities discover peers, register, export root hashes etc...
    use anyhow::{bail, Context, Result};
    use log::{debug, error, info, warn};

    use reqwest::{Client, StatusCode, Url};

    #[derive(Debug)]
    pub struct Peer {
        pub name: String,
        pub addresses: Vec<String>,
    }

    /// Parse newline separated data into an array of strings.
    pub fn parse_newline_separated(data: impl Into<String>) -> Vec<String> {
        let data = data.into();
        let tokens = data
            .split('\n')
            .filter_map(|x| match x {
                "" => None,
                _ => Some(x.to_owned()),
            })
            .collect();
        return tokens;
    }

    /// Attempt to get data from a host and return it as a String.
    pub async fn get_raw_data(client: Client, url: &Url) -> Result<String> {
        let request = client.get(url.clone());
        let response = request
            .send()
            .await
            .context(format!("Failed to connect to host {}.", url.as_str()))?;

        if response.status() != StatusCode::OK {
            bail!(
                "Failed to get data from host {} with status code {}.",
                response.url().as_str(),
                response.status()
            );
        } else {
            match response.text().await {
                Ok(s) => return Ok(s),
                Err(e) => bail!(format!(
                    "Faile to get retrieve text from host {}",
                    url.as_str()
                )),
            }
        }
    }

    pub async fn get_peer_addresses(client: Client, url: &Url, peer: &str) -> Result<Peer> {
        let data = get_raw_data(client, url).await?;
        return Ok(Peer {
            name: peer.to_string(),
            addresses: parse_newline_separated(data),
        });
    }

    pub async fn get_peers(client: Client, url: &Url) -> Result<Vec<String>> {
        let data = get_raw_data(client, url).await?;
        let peers = parse_newline_separated(data);

        return Ok(peers);
    }

    #[cfg(test)]
    mod tests {

        use super::*;
        use std::time::Duration;
        use tokio;

        #[tokio::test]
        async fn lib_web_get_peers() {
            let host = Url::parse("https://jch.irif.fr:8443/peers").unwrap();
            let five_seconds = Duration::new(5, 0);
            let client = Client::builder()
                .timeout(five_seconds)
                .user_agent("Projet M2 protocoles Internet")
                .build()
                .unwrap();
            let result = get_peers(client, &host).await.unwrap();
            println!("Result = {:?}", result);
        }

        #[tokio::test]
        async fn lib_web_get_peer_addresses() {
            let host = Url::parse("https://jch.irif.fr:8443/peers/jch.irif.fr/addresses").unwrap();
            let five_seconds = Duration::new(5, 0);
            let client = Client::builder()
                .timeout(five_seconds)
                .user_agent("Projet M2 protocoles Internet")
                .build()
                .unwrap();
            let result = get_peer_addresses(client, &host, "jch.irif.fr")
                .await
                .unwrap();
            println!("Result = {:?}", result);
        }
    }
}
