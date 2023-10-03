// TODO
// re-make asyc version (no image downloading)

// use async_recursion::async_recursion;
mod write;
use crate::write::article_to_disk;
use crate::write::title_page_to_disk;

mod data;
use crate::data::Article;
use crate::data::FeedMetadata;

mod config;
use crate::config::*;

mod build;
use crate::build::generate_epub;

use std::error::Error;

use rss::Channel;

fn get_feed(page: &u8) -> Result<Channel, Box<dyn Error>> {
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

fn paginate_feed(
    page: u8,
    mut articles: Option<Vec<Article>>,
    feed_metadata: &FeedMetadata,
) -> Result<(), color_eyre::Report> {
    let current_page = &page;
    let res = get_feed(current_page);

    let items = &res.unwrap().items;

    match items.len() {
        0 => {
            let feed_title: &str = &feed_metadata.title;
            let feed_description: &str = &feed_metadata.description;

            return generate_epub(feed_title, feed_description, articles);
        }
        _ => {
            let new_articles = article_to_disk(items);
            match articles {
                None => articles = new_articles,
                Some(_) => articles.as_mut().unwrap().extend(new_articles.unwrap()),
            }
        }
    }
    let next_page = current_page + 1;
    paginate_feed(next_page, articles, feed_metadata)
}

fn main() -> Result<(), Box<dyn Error>> {
    let feed_metadata = get_feed_metadata()?;
    title_page_to_disk(&feed_metadata)?;

    Ok(paginate_feed(STARTING_PAGE, None, &feed_metadata)?)
}
