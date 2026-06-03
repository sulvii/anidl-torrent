use crate::http::client::HttpClient;
use anyhow::{Context, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://anidb.net";
const SEARCH_URL: &str = "https://anidb.net/search/anime/";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub score: f32,
    pub entity_type: String,
    pub title: String,
    pub url: String,
    pub id: Option<u32>,
    pub excerpt: String,
    pub thumbnail: Option<String>,
}

pub async fn anidb_search(client: &HttpClient, query: &str) -> Result<Vec<SearchResult>> {
    let html: String = client
        .inner
        .get(SEARCH_URL)
        .query(&[
            ("adb.search", query),
            ("do.search", "1"),
            ("entity.animetb", "1"),
            ("field.titles", "1"),
        ] as &[(&str, &str)])
        .send()
        .await
        .context("request failed")?
        .error_for_status()
        .context("bad status")?
        .text()
        .await
        .context("body read failed")?;

    parse_results(&html)
}

fn parse_results(html: &str) -> Result<Vec<SearchResult>> {
    let document = Html::parse_document(html);

    let row_sel = Selector::parse("table.search_results tbody tr").unwrap();
    let score_sel = Selector::parse("td.score").unwrap();
    let type_sel = Selector::parse("td.type").unwrap();
    let relid_sel = Selector::parse("td.relid a").unwrap();
    let excerpt_sel = Selector::parse("td.excerpt").unwrap();
    let img_sel = Selector::parse("td.thumb img").unwrap();

    let mut results = Vec::new();

    for row in document.select(&row_sel) {
        let score: f32 = row
            .select(&score_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default()
            .parse()
            .unwrap_or(0.0);

        let entity_type = row
            .select(&type_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let Some(anchor) = row.select(&relid_sel).next() else {
            continue;
        };
        let title = anchor.text().collect::<String>().trim().to_owned();
        if title.is_empty() {
            continue;
        }

        let href = anchor.value().attr("href").unwrap_or_default();
        let url = if href.starts_with("http") {
            href.to_owned()
        } else {
            format!("{BASE_URL}{href}")
        };
        let id: Option<u32> = href
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .and_then(|s| s.parse().ok());

        let excerpt = row
            .select(&excerpt_sel)
            .next()
            .map(|el| {
                el.text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default();

        let thumbnail = row
            .select(&img_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(|s| s.to_owned());

        results.push(SearchResult {
            score,
            entity_type,
            title,
            url,
            id,
            excerpt,
            thumbnail,
        });
    }

    Ok(results)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: Option<u32>,
    pub number: String,
    pub ep_type: String,
    pub title: String,
    pub duration: Option<String>,
    pub air_date: Option<String>,
    pub url: Option<String>,
    pub stream_url: Option<String>,
}

pub async fn get_episodes(client: &HttpClient, anime_id: u32) -> Result<Vec<Episode>> {
    let url = format!("{BASE_URL}/anime/{anime_id}");
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

    parse_episodes(&html)
}

fn parse_episodes(html: &str) -> Result<Vec<Episode>> {
    let document = Html::parse_document(html);

    let row_sel = Selector::parse("table#eplist tbody tr").unwrap();
    let eid_sel = Selector::parse("td.id.eid a").unwrap();
    let abbr_sel = Selector::parse("td.id.eid a abbr").unwrap();
    let title_sel = Selector::parse("td.title.name.episode label").unwrap();
    let dur_sel = Selector::parse("td.duration").unwrap();
    let date_sel = Selector::parse("td.date.airdate").unwrap();
    let stream_sel = Selector::parse("td.action.episode a").unwrap();

    let mut episodes = Vec::new();

    for row in document.select(&row_sel) {
        let (number, ep_type) = row
            .select(&abbr_sel)
            .next()
            .map(|abbr| {
                let num = abbr.text().collect::<String>().trim().to_owned();
                let typ = abbr.value().attr("title").unwrap_or("").trim().to_owned();
                (num, typ)
            })
            .unwrap_or_default();

        if number.is_empty() {
            continue;
        }

        let url = row
            .select(&eid_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(|href| {
                if href.starts_with("http") {
                    href.to_owned()
                } else {
                    format!("{BASE_URL}{href}")
                }
            });

        let id: Option<u32> = url
            .as_deref()
            .and_then(|u| u.trim_end_matches('/').rsplit('/').next())
            .and_then(|s| s.parse().ok());

        let title = row
            .select(&title_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .unwrap_or_default();

        let duration = row
            .select(&dur_sel)
            .next()
            .map(|el| el.text().collect::<String>().trim().to_owned())
            .filter(|s| !s.is_empty());

        let air_date = row
            .select(&date_sel)
            .next()
            .map(|el| {
                el.value()
                    .attr("content")
                    .map(str::to_owned)
                    .unwrap_or_else(|| el.text().collect::<String>().trim().to_owned())
            })
            .filter(|s| !s.is_empty());

        let stream_url = row
            .select(&stream_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(str::to_owned);

        episodes.push(Episode {
            id,
            number,
            ep_type,
            title,
            duration,
            air_date,
            url,
            stream_url,
        });
    }

    Ok(episodes)
}
