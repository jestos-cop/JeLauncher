use serde::{Deserialize, Serialize};

use super::MetaService;
use crate::Result;

const NEWS_HOST: &str = "https://launchercontent.mojang.com";
const NEWS_URL: &str = "https://launchercontent.mojang.com/v2/news.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct NewsEntry {
    pub id: String,
    pub title: String,
    pub text: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    pub url: String,
    pub image_url: Option<String>,
}

#[derive(Deserialize)]
struct NewsFeed {
    entries: Vec<FeedEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FeedEntry {
    id: String,
    title: String,
    #[serde(default)]
    text: String,
    date: String,
    read_more_link: String,
    #[serde(default)]
    news_type: Vec<String>,
    news_page_image: Option<FeedImage>,
}

#[derive(Deserialize)]
struct FeedImage {
    url: String,
}

impl MetaService {
    /// Minecraft: Java Edition news from Mojang, newest first.
    pub async fn news(&self) -> Result<Vec<NewsEntry>> {
        let cache = self.cache_dir.join("news.json");
        let feed: NewsFeed = self.client.get_json_cached(NEWS_URL, &cache).await?;
        Ok(java_news(feed))
    }
}

fn java_news(feed: NewsFeed) -> Vec<NewsEntry> {
    let mut entries: Vec<NewsEntry> = feed
        .entries
        .into_iter()
        .filter(|entry| entry.news_type.iter().any(|kind| kind == "Java"))
        .map(|entry| NewsEntry {
            id: entry.id,
            title: entry.title.trim().to_owned(),
            text: entry.text.trim().to_owned(),
            date: entry.date,
            url: entry.read_more_link,
            image_url: entry
                .news_page_image
                .map(|image| format!("{NEWS_HOST}{}", image.url)),
        })
        .collect();
    entries.sort_by(|a, b| b.date.cmp(&a.date));
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_java_news_newest_first() -> std::result::Result<(), serde_json::Error> {
        let feed: NewsFeed = serde_json::from_str(
            r#"{"version":1,"entries":[
                {"id":"1","title":"Old ","text":"a","date":"2026-01-01","readMoreLink":"https://x/1","newsType":["News page","Java"]},
                {"id":"2","title":"Bedrock","text":"b","date":"2026-05-01","readMoreLink":"https://x/2","newsType":["Bedrock"]},
                {"id":"3","title":"New","text":"c","date":"2026-09-01","readMoreLink":"https://x/3","newsType":["Java"],
                 "newsPageImage":{"url":"/v2/images/new.png"}}]}"#,
        )?;
        let news = java_news(feed);
        let titles: Vec<_> = news.iter().map(|entry| entry.title.as_str()).collect();
        assert_eq!(titles, ["New", "Old"]);
        assert_eq!(
            news[0].image_url.as_deref(),
            Some("https://launchercontent.mojang.com/v2/images/new.png")
        );
        Ok(())
    }
}
