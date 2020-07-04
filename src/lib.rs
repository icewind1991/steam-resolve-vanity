use reqwest::{Client, StatusCode};
use serde::Deserialize;
pub use steamid_ng::SteamID;
use thiserror::Error;

#[derive(Deserialize)]
struct SteamApiResponse {
    response: VanityUrlResponse,
}

#[derive(Deserialize)]
struct VanityUrlResponse {
    #[serde(default)]
    steamid: String,
    success: u8,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid api key")]
    InvalidKey,
    #[error("Error while making request to steam api")]
    Request(#[from] reqwest::Error),
    #[error("Received malformed steam id")]
    SteamId(#[from] steamid_ng::SteamIDParseError),
}

/// Resolve a steam vanity url to a steam id
pub async fn resolve_vanity_url(url: &str, api_key: &str) -> Result<Option<SteamID>, Error> {
    let response: reqwest::Response = Client::new()
        .get("http://api.steampowered.com/ISteamUser/ResolveVanityURL/v0001/")
        .query(&[("key", api_key), ("vanityurl", url)])
        .send()
        .await?;

    if response.status() == StatusCode::FORBIDDEN {
        return Err(Error::InvalidKey);
    }

    let api_response: SteamApiResponse = response.json().await?;

    if api_response.response.success == 1 {
        let steam_id: SteamID = api_response.response.steamid.parse()?;

        Ok(Some(steam_id))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_valid() {
    let key = dotenv::var("STEAM_API_KEY").unwrap();
    assert_eq!(
        Some(SteamID::from(76561198024494988)),
        resolve_vanity_url("icewind1991", &key).await.unwrap()
    )
}

#[cfg(test)]
#[tokio::test]
async fn test_invalid_key() {
    assert!(matches!(
        resolve_vanity_url("icewind1991", "foo").await.unwrap_err(),
        Error::InvalidKey
    ))
}

#[cfg(test)]
#[tokio::test]
async fn test_not_found() {
    let key = dotenv::var("STEAM_API_KEY").unwrap();
    assert_eq!(
        None,
        resolve_vanity_url("hopefully_non_existing_steam_id", &key)
            .await
            .unwrap()
    )
}
