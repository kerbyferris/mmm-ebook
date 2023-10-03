// TODO
// re-make async version (no image downloading)

// use async_recursion::async_recursion;
mod write;
use crate::write::article_to_disk;
use crate::write::title_page_to_disk;

mod data;
use crate::data::Article;
use crate::data::ArticleOnDisk;
use crate::data::FeedMetadata;

mod config;
use crate::config::*;

mod build;
use crate::build::generate_epub;

use std::error::Error;

use chrono::DateTime;

use rss::Channel;

fn get_feed(page: &u16) -> Result<Channel, Box<dyn Error>> {
    let url = format!("{}{}{}", RSS_URL, QUERY, page);

    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;

    Ok(channel)
}

fn get_feed_metadata() -> Result<FeedMetadata, Box<dyn Error>> {
    let content = reqwest::blocking::get(RSS_URL)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;

    let feed_metadata = FeedMetadata {
        title: channel.title,
        description: channel.description,
    };

    Ok(feed_metadata)
}

fn rss_item_to_struct(item: rss::Item) -> Option<Article> {
    let time_stamp = DateTime::parse_from_rfc2822(&item.pub_date.unwrap().as_ref()).unwrap();
    let date = time_stamp.format("%b %d, %Y").to_string();

    Some(Article {
        title: item.title.unwrap(),
        date,
        time_stamp,
        content: item.content.unwrap(),
    })
}

fn handle_page(page_num: u16) -> Result<Option<Vec<Article>>, Box<dyn Error>> {
    let items = get_feed(&page_num).unwrap().items;

    if items.is_empty() {
        Ok(None)
    } else {
        let articles = items
            .iter()
            .map(|i| rss_item_to_struct(i.clone()))
            .collect();

        Ok(articles)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let feed_metadata = get_feed_metadata()?;
    title_page_to_disk(&feed_metadata)?;

    let data = (1..)
        .map_while(|n| handle_page(n).expect("Error retrieving RSS feed"))
        .collect::<Vec<_>>();

    let articles: Vec<Article> = data.into_iter().flatten().collect();
    let articles_on_disk: Vec<ArticleOnDisk> = articles
        .iter()
        .map(|a| article_to_disk(a.clone()).unwrap())
        .collect();

    generate_epub(
        &feed_metadata.title,
        &feed_metadata.description,
        articles_on_disk,
    )?;
    Ok(())
}
