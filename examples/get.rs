use std::convert::TryFrom;
use std::env::args;
use steam_resolve_vanity::get_vanity_url;
use steamid_ng::SteamID;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let binary = args.next().unwrap(); // first argument is binary

    if let Some(steam_id) = args.next() {
        if let Some(vanity) = get_vanity_url(SteamID::try_from(steam_id.as_str())?).await? {
            println!("{}", vanity);
        } else {
            println!("No vanity found for steamid");
        }
    } else {
        eprintln!("usage {} <vanity>", binary);
    }

    Ok(())
}
