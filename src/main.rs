mod http;
mod providers;

use anyhow::Result;
use http::client::HttpClient;
use providers::animetosho::get_torrent_info;

#[tokio::main]
async fn main() -> Result<()> {
    let client = HttpClient::new()?;

    let info = get_torrent_info(&client, 608521).await?;
    println!("Title:     {}", info.title);
    println!("Hash:      {}", info.info_hash.as_deref().unwrap_or("—"));
    println!("Size:      {:?} bytes", info.size_bytes);
    println!(
        "Submitted: {}",
        info.date_submitted.as_deref().unwrap_or("—")
    );
    println!("Category:  {}", info.category.as_deref().unwrap_or("—"));
    println!("File:      {:?}", info.file_name);
    println!("Trackers ({}):", info.trackers.len());
    for t in &info.trackers {
        println!("  {t}");
    }
    println!("DDL ({}):", info.ddl_links.len());
    for d in &info.ddl_links {
        println!("  {:12}  {}", d.provider, d.url);
    }
    println!("Extractions ({}):", info.extraction_links.len());
    for e in &info.extraction_links {
        println!("  {:12}  {}", e.provider, e.url);
    }
    println!("Subtitles ({}):", info.subtitle_tracks.len());
    for s in &info.subtitle_tracks {
        println!("  {}", s.label);
    }
    println!("Screenshots ({}):", info.screenshots.len());
    for s in &info.screenshots {
        println!("  {s}");
    }

    Ok(())
}
