mod write;
use crate::write::article_to_disk;
use crate::write::title_page_to_disk;
use crate::write::title_page_to_disk;

mod data;
use crate::data::Article;
use crate::data::ArticleOnDisk;
use crate::data::FeedMetadata;

mod config;
use crate::config::*;

mod build;
use crate::build::generate_epub;

use color_eyre::eyre;

use chrono::DateTime;

use rss::Channel;

use rayon::prelude::*;

fn get_feed_page(page: &u16) -> Result<Channel, eyre::Report> {
    let url = format!("{}{}{}", RSS_URL, QUERY, page);

    let content = reqwest::blocking::get(url)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;

    Ok(channel)
}

fn get_feed_metadata() -> Result<FeedMetadata, eyre::Report> {
    let content = reqwest::blocking::get(RSS_URL)?.bytes()?;
    let channel = Channel::read_from(&content[..])?;

    let feed_metadata = FeedMetadata {
        title: channel.title,
        description: channel.description,
    };

    Ok(feed_metadata)
}

fn rss_item_to_struct(item: rss::Item) -> Result<Option<Article>, eyre::Report> {
    let timestamp = DateTime::parse_from_rfc2822(&item.pub_date.unwrap().as_ref()).unwrap();
    let date = timestamp.format("%b %d, %Y").to_string();

    Ok(Some(Article {
        title: item.title.unwrap(),
        date,
        timestamp,
        content: item.content.unwrap(),
    }))
}

fn handle_page(page_num: u16) -> Result<Option<Vec<Article>>, eyre::Report> {
    let items = get_feed_page(&page_num)?.items;

    if items.is_empty() {
        Ok(None)
    } else {
        let articles = items
            .par_iter()
            .map(|i| rss_item_to_struct(i.clone()).unwrap())
            .collect();

        Ok(articles)
    }
}

fn main() -> Result<(), eyre::Report> {
    let build_epub = true; // TODO

    let feed_metadata = get_feed_metadata()?;
    title_page_to_disk(&feed_metadata)?;

    let data = (1..)
        .map_while(|n| handle_page(n).expect("Error retrieving RSS feed"))
        .collect::<Vec<_>>();

    let articles: Vec<Article> = data.into_iter().flatten().collect();
    let articles_on_disk: Vec<ArticleOnDisk> = articles
        .par_iter()
        .map(|a| article_to_disk(a.clone()).unwrap().unwrap())
        .collect();

    if build_epub {
        generate_epub(
            &feed_metadata.title,
            &feed_metadata.description,
            articles_on_disk,
        )?;
    }

    Ok(())
}
