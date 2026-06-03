mod http;
mod providers;

use anyhow::Result;
use http::client::HttpClient;
use providers::animetosho::get_torrents;

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::new()?;

    let releases = get_torrents(&client, 310799).await?;
    for r in &releases {
        println!(
            "[{}] {:>6}↑/{:<6}↓  {}",
            r.id,
            r.seeders
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into()),
            r.leechers
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into()),
            r.title,
        );
        for ddl in &r.ddl_links {
            println!("      {:12}  {}", ddl.provider, ddl.url);
        }
    }
    println!("\n{} releases", releases.len());

    Ok(())
}
