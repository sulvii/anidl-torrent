mod http;
mod providers;

use anyhow::Result;
use http::client::HttpClient;
use providers::anidb::{anidb_search, get_episodes};

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::new()?;

    let episodes = get_episodes(&client, 18209).await?;
    for ep in &episodes {
        println!(
            "{:<4} [{:<22}] {:>4} {:#?}  {}  {}",
            ep.number,
            ep.ep_type,
            ep.duration.as_deref().unwrap_or("—"),
            ep.id,
            ep.air_date.as_deref().unwrap_or("—"),
            ep.title,
        );
    }
    println!("\n{} episodes", episodes.len());
    Ok(())
}
