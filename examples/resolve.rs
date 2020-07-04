use std::env::args;
use steam_resolve_vanity::resolve_vanity_url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = dotenv::var("STEAM_API_KEY")?;
    let mut args = args();
    let binary = args.next().unwrap(); // first argument is binary

    if let Some(vanity) = args.next() {
        if let Some(steam_id) = resolve_vanity_url(&vanity, &key).await? {
            println!("{}", steam_id.steam3());
        } else {
            println!("No steamid found for vanity");
        }
    } else {
        eprintln!("usage {} <vanity>", binary);
    }

    Ok(())
}
