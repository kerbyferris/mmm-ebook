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

use std::error::Error;
use std::fs::File;

// use epub_builder::TocElement;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipCommand};
use rss::Channel;

use std::env::current_dir;

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

fn paginate_feed(page: u8, mut articles: Option<Vec<Article>>, feed_metadata: &FeedMetadata) {
    let current_page = &page;
    let res = get_feed(current_page);

    let items = &res.unwrap().items;

    match items.len() {
        0 => {
            let feed_title: &str = &feed_metadata.title;
            let feed_description: &str = &feed_metadata.description;

            build_epub(feed_title, feed_description, articles);

            return;
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
    paginate_feed(next_page, articles, feed_metadata);
}

fn build_epub(title: &str, description: &str, articles: Option<Vec<Article>>) {
    let mut builder = EpubBuilder::new(ZipCommand::new().unwrap()).unwrap();
    builder.metadata("title", title).unwrap();
    builder.metadata("title", description).unwrap();
    builder
        .stylesheet(File::open(format!("{}/style.css", &ASSETS_DIR)).unwrap())
        .unwrap();
    builder
        .add_cover_image(
            "cover.png",
            File::open(format!("{}/cover.png", &ASSETS_DIR)).unwrap(),
            "image/png",
        )
        .unwrap();
    builder
        .add_content(
            EpubContent::new("cover.xhtml", title.to_string().as_bytes())
                .reftype(ReferenceType::Cover),
        )
        .unwrap();
    builder
        .add_content(
            EpubContent::new(
                "title.xhtml",
                File::open(format!("{}/title.html", &OUTPUT_HTML_DIR)).unwrap(),
            )
            .title(title)
            .reftype(epub_builder::ReferenceType::TitlePage),
        )
        .unwrap();
    builder.inline_toc();
    builder.set_toc_name(description);
    for article in articles.unwrap() {
        builder
            .add_content(
                EpubContent::new(
                    article.chapter_title,
                    File::open(article.file_path).unwrap(),
                )
                .title(article.title)
                .reftype(ReferenceType::Text),
            )
            .unwrap();
        for image in article.images {
            builder
                .add_resource(image.name, File::open(image.path).unwrap(), image.mime_type)
                .unwrap();
        }
    }
    let curr_dir = current_dir().expect("no current directory");
    let temp_file = curr_dir.join(format!("{}/{}", &OUTPUT_DIR, &EPUB_NAME));

    let mut file = File::create(temp_file).expect("no file");

    builder.generate(&mut file).expect("no epub");
}

fn main() {
    let feed_metadata = get_feed_metadata().unwrap();
    title_page_to_disk(&feed_metadata).unwrap();

    paginate_feed(STARTING_PAGE, None, &feed_metadata);
}
