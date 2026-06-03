use crate::http::client::HttpClient;
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://animetosho.xyz";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentRelease {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub torrent_url: Option<String>,
    pub magnet_url: Option<String>,
    pub nzb_url: Option<String>,
    pub seeders: Option<u32>,
    pub leechers: Option<u32>,
    pub size_bytes: Option<u64>,
    pub ddl_links: Vec<DdlLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdlLink {
    pub provider: String,
    pub url: String,
}

pub async fn get_torrents(client: &HttpClient, episode_id: u32) -> Result<Vec<TorrentRelease>> {
    let url = format!("{BASE_URL}/episode/{episode_id}");
    let html = client
        .inner
        .get(&url)
        .send()
        .await
        .context("request failed")?
        .error_for_status()
        .context("bad status")?
        .text()
        .await
        .context("body read failed")?;

    parse_torrents(&html)
}

fn parse_torrents(html: &str) -> Result<Vec<TorrentRelease>> {
    let document = Html::parse_document(html);

    let entry_sel = Selector::parse("div.home_list_entry").unwrap();
    let title_sel = Selector::parse("div.link a").unwrap();
    let size_sel = Selector::parse("div.size").unwrap();
    let torrent_sel = Selector::parse("div.links a.dllink").unwrap();
    let magnet_sel = Selector::parse("div.links a[href^='magnet:']").unwrap();
    let nzb_sel = Selector::parse("div.links a[href$='.nzb.gz']").unwrap();
    let peers_sel = Selector::parse("div.links span[title]").unwrap();
    let ddl_sel =
        Selector::parse("div.links a:not(.dllink):not([href^='magnet:']):not([href$='.nzb.gz'])")
            .unwrap();

    let mut releases = Vec::new();

    for entry in document.select(&entry_sel) {
        let Some(title_anchor) = entry.select(&title_sel).next() else {
            continue;
        };
        let title = title_anchor
            .value()
            .attr("title")
            .unwrap_or_else(|| {
                title_anchor
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_owned()
                    .leak()
            })
            .to_owned();

        let href = title_anchor.value().attr("href").unwrap_or_default();
        let url = if href.starts_with("http") {
            href.to_owned()
        } else {
            format!("{BASE_URL}{href}")
        };

        let id: u32 = url
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let size_bytes: Option<u64> = entry
            .select(&size_sel)
            .next()
            .and_then(|el| el.value().attr("title"))
            .and_then(|t| t.split_whitespace().nth(3))
            .and_then(|s| s.parse().ok());

        let torrent_url = entry
            .select(&torrent_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_owned()
                } else {
                    format!("{BASE_URL}{h}")
                }
            });

        let magnet_url = entry
            .select(&magnet_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(str::to_owned);

        let nzb_url = entry
            .select(&nzb_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|h| {
                if h.starts_with("http") {
                    h.to_owned()
                } else {
                    format!("{BASE_URL}{h}")
                }
            });

        let (seeders, leechers) = entry
            .select(&peers_sel)
            .find_map(|el| {
                let t = el.value().attr("title")?;
                parse_peers(t)
            })
            .unwrap_or((None, None));

        let ddl_links: Vec<DdlLink> = entry
            .select(&ddl_sel)
            .filter_map(|a| {
                let href = a.value().attr("href")?;
                if href.starts_with('/') {
                    return None;
                }
                let label = a.text().collect::<String>().trim().to_owned();
                if label.is_empty() {
                    return None;
                }
                Some(DdlLink {
                    provider: label,
                    url: href.to_owned(),
                })
            })
            .collect();

        releases.push(TorrentRelease {
            id,
            title,
            url,
            torrent_url,
            magnet_url,
            nzb_url,
            seeders,
            leechers,
            size_bytes,
            ddl_links,
        });
    }

    Ok(releases)
}

fn parse_peers(title: &str) -> Option<(Option<u32>, Option<u32>)> {
    if !title.contains("Seeders") {
        return None;
    }
    let mut parts = title.splitn(2, '/');
    let seeders = parts.next()?.split_whitespace().nth(1)?.parse().ok();
    let leechers = parts.next()?.split_whitespace().nth(1)?.parse().ok();
    Some((seeders, leechers))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleTrack {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub torrent_url: Option<String>,
    pub magnet_url: Option<String>,
    pub nzb_url: Option<String>,
    pub info_hash: Option<String>,
    pub date_submitted: Option<String>,
    pub category: Option<String>,
    pub size_bytes: Option<u64>,
    pub file_name: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub trackers: Vec<String>,
    pub ddl_links: Vec<DdlLink>,
    pub extraction_links: Vec<DdlLink>,
    pub subtitles_all_url: Option<String>,
    pub subtitle_tracks: Vec<SubtitleTrack>,
    pub screenshots: Vec<String>,
}

pub async fn get_torrent_info(client: &HttpClient, release_id: u32) -> Result<TorrentInfo> {
    let url = format!("{BASE_URL}/view/{release_id}");
    let html = client
        .inner
        .get(&url)
        .send()
        .await
        .context("request failed")?
        .error_for_status()
        .context("bad status")?
        .text()
        .await
        .context("body read failed")?;

    parse_torrent_info(&html, release_id)
}

fn parse_torrent_info(html: &str, release_id: u32) -> Result<TorrentInfo> {
    let document = Html::parse_document(html);

    let title_sel = Selector::parse("h2#title").unwrap();
    let row_sel = Selector::parse("table tr").unwrap();
    let th_sel = Selector::parse("th").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let tracker_sel = Selector::parse("td.tracker_url").unwrap();
    let screenshot_sel = Selector::parse("a.screenthumb").unwrap();
    let anchor_sel = Selector::parse("a").unwrap();
    let code_sel = Selector::parse("code").unwrap();

    let url = format!("{BASE_URL}/view/{release_id}");

    let title = document
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .unwrap_or_default();

    let screenshots: Vec<String> = document
        .select(&screenshot_sel)
        .filter_map(|a| a.value().attr("href").map(str::to_owned))
        .collect();

    let trackers: Vec<String> = document
        .select(&tracker_sel)
        .map(|el| el.text().collect::<String>().trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect();

    let mut torrent_url = None::<String>;
    let mut magnet_url = None::<String>;
    let mut nzb_url = None::<String>;
    let mut info_hash = None::<String>;
    let mut date_submitted = None::<String>;
    let mut category = None::<String>;
    let mut size_bytes = None::<u64>;
    let mut file_name = None::<String>;
    let mut file_size_bytes = None::<u64>;
    let mut ddl_links = Vec::<DdlLink>::new();
    let mut extraction_links = Vec::<DdlLink>::new();
    let mut subtitles_all_url = None::<String>;
    let mut subtitle_tracks = Vec::<SubtitleTrack>::new();

    for row in document.select(&row_sel) {
        let Some(th) = row.select(&th_sel).next() else {
            continue;
        };
        let Some(td) = row.select(&td_sel).next() else {
            continue;
        };

        let label = th.text().collect::<String>().trim().to_owned();

        match label.as_str() {
            "Source Links" => {
                for a in td.select(&anchor_sel) {
                    let href = a.value().attr("href").unwrap_or_default();
                    let text = a.text().collect::<String>().trim().to_owned();
                    if href.starts_with("magnet:") {
                        magnet_url = Some(href.to_owned());
                    } else if href.ends_with(".nzb.gz") {
                        nzb_url = Some(abs(href));
                    } else if text == "Torrent Download" {
                        torrent_url = Some(href.to_owned());
                    }
                }
            }

            "Date Submitted" => {
                date_submitted = Some(td.text().collect::<String>().trim().to_owned());
            }

            "Category" => {
                category = Some(td.text().collect::<String>().trim().to_owned());
            }

            "Info Hash" => {
                info_hash = td
                    .select(&code_sel)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_owned());
            }

            "File Name (Size)" => {
                file_name = td
                    .select(&anchor_sel)
                    .next()
                    .map(|a| a.text().collect::<String>().trim().to_owned());

                let span_sel = Selector::parse("span[title]").unwrap();
                file_size_bytes = td
                    .select(&span_sel)
                    .next()
                    .and_then(|s| s.value().attr("title"))
                    .and_then(|t| t.split_whitespace().nth(2))
                    .and_then(|s| s.parse().ok());
            }

            "Download" => {
                ddl_links = collect_ddl_links(&td);
            }

            "Extractions" => {
                extraction_links = collect_ddl_links(&td);
            }

            "Subtitles" => {
                for a in td.select(&anchor_sel) {
                    let href = a.value().attr("href").unwrap_or_default();
                    let text = a.text().collect::<String>().trim().to_owned();
                    if text == "All Attachments" {
                        subtitles_all_url = Some(abs(href));
                    } else {
                        subtitle_tracks.push(SubtitleTrack {
                            label: text,
                            url: abs(href),
                        });
                    }
                }
            }

            _ => {}
        }
    }

    let jsonld_sel = Selector::parse("script[type='application/ld+json']").unwrap();
    if let Some(script) = document.select(&jsonld_sel).next() {
        let json_text = script.text().collect::<String>();
        size_bytes = extract_json_u64(&json_text, "fileSize");
    }

    Ok(TorrentInfo {
        id: release_id,
        title,
        url,
        torrent_url,
        magnet_url,
        nzb_url,
        info_hash,
        date_submitted,
        category,
        size_bytes,
        file_name,
        file_size_bytes,
        trackers,
        ddl_links,
        extraction_links,
        subtitles_all_url,
        subtitle_tracks,
        screenshots,
    })
}

fn abs(href: &str) -> String {
    if href.starts_with("http") {
        href.to_owned()
    } else {
        format!("{BASE_URL}{href}")
    }
}

fn collect_ddl_links(td: &scraper::ElementRef) -> Vec<DdlLink> {
    let anchor_sel = Selector::parse("a").unwrap();
    td.select(&anchor_sel)
        .filter_map(|a| {
            let href = a.value().attr("href")?;
            if href.starts_with('/') {
                return None;
            }
            let provider = a.text().collect::<String>().trim().to_owned();
            if provider.is_empty() {
                return None;
            }
            Some(DdlLink {
                provider,
                url: href.to_owned(),
            })
        })
        .collect()
}

fn extract_json_u64(json: &str, key: &str) -> Option<u64> {
    let needle = format!("\"{}\":", key);
    let start = json.find(&needle)? + needle.len();
    let rest = json[start..].trim_start_matches('"').trim_start();
    rest.split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}
