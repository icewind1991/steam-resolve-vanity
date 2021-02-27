use reqwest::header::LOCATION;
use reqwest::redirect::Policy;
use reqwest::{Client, ClientBuilder, StatusCode};
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
    steamid: Option<SteamID>,
    success: u8,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid api key")]
    InvalidKey,
    #[error("Error while making request to steam api")]
    Request(#[from] reqwest::Error),
    #[error("Received malformed steam id")]
    SteamId(#[from] steamid_ng::SteamIDError),
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

    Ok(api_response
        .response
        .steamid
        .filter(|_| api_response.response.success == 1))
}

pub async fn get_vanity_url(steam_id: SteamID) -> Result<Option<String>, Error> {
    let response: reqwest::Response = ClientBuilder::new()
        .redirect(Policy::none())
        .build()
        .unwrap()
        .get(&format!(
            "https://steamcommunity.com/profiles/{}",
            u64::from(steam_id)
        ))
        .send()
        .await?;

    Ok(match response.status() {
        StatusCode::FOUND => response
            .headers()
            .into_iter()
            .find_map(|(name, value)| if name == LOCATION { Some(value) } else { None })
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split_at("https://steamcommunity.com/id/".len()).1)
            .map(|value| value.trim_end_matches("/").to_string()),
        _ => None,
    })
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

#[cfg(test)]
#[tokio::test]
async fn test_get_vanity() {
    assert_eq!(
        Some("icewind1991".to_string()),
        get_vanity_url(SteamID::from(76561198024494988))
            .await
            .unwrap()
    )
}

#[cfg(test)]
#[tokio::test]
async fn test_get_vanity_not_found() {
    assert_eq!(
        None,
        get_vanity_url(SteamID::from(76561198024494987))
            .await
            .unwrap()
    )
}
