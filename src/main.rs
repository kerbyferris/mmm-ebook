use async_recursion::async_recursion;
use chrono::DateTime;
use chrono::Datelike;
// use epub_builder::TocElement;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, Result, ZipCommand};
use rss::Channel;
use std::error::Error;

use build_html::*;

use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

const RSS_URL: &str = "http://www.mrmoneymustache.com/feed/";
const QUERY: &str = "?order=ASC&paged=";
const OUTPUT_DIR: &str = "output";
const OUTPUT_HTML_DIR: &str = "output/html";

#[derive(Debug)]
struct Article {
    title: String,
    file_path: String,
    chapter_title: String,
}

#[derive(Debug)]
struct FeedMetadata {
    title: String,
    description: String,
}

async fn get_feed(page: &u8) -> Result<Channel, Box<dyn Error>> {
    let url = format!("{}{}{}", RSS_URL, QUERY, page.to_string());

    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    Ok(channel)
}

async fn get_feed_metadata() -> Result<FeedMetadata, Box<dyn Error>> {
    let content = reqwest::get(RSS_URL).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    let feed_metadata = FeedMetadata {
        title: channel.title,
        description: channel.description,
    };

    Ok(feed_metadata)
}

fn write_title_page_to_disk(feed_metadata: &FeedMetadata) {
    let page = build_html::HtmlPage::new()
        .with_header(1, &feed_metadata.title)
        .with_paragraph(&feed_metadata.description)
        .to_html_string();

    let file_path = format!("{}/title.html", OUTPUT_DIR);
    let mut file = fs::File::create(&file_path).unwrap();
    file.write_all(page.as_bytes()).unwrap();
}

fn write_article_to_disk(items: &Vec<rss::Item>) -> Option<Vec<Article>> {
    let mut articles: Vec<Article> = vec![];
    for item in items {
        let date = DateTime::parse_from_rfc2822(item.pub_date.as_ref().unwrap()).unwrap();

        let year = date.year().to_string();
        let date_string = date.format("%b %d, %Y").to_string();
        let date_path_string = date.format("%Y%m%d").to_string();

        let title = item.title.as_ref().unwrap();
        let content = item.content.as_ref().unwrap();

        let output_dir = format!("{}/{}", &OUTPUT_HTML_DIR, &year);

        let page = build_html::HtmlPage::new()
            .with_paragraph(&date_string)
            .with_header(1, title)
            .with_paragraph(content)
            .to_html_string();

        fs::create_dir_all(&output_dir).unwrap();

        let chapter_title = format!("chapter_{}.html", &date_path_string);

        let file_path = format!("{}/{}", &output_dir, &chapter_title);

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(page.as_bytes()).unwrap();

        let article = Article {
            title: title.to_string(),
            file_path,
            chapter_title,
        };
        articles.push(article)
    }
    return Some(articles);
}

#[async_recursion(?Send)]
async fn paginate_feed(page: u8, mut articles: Option<Vec<Article>>, feed_metadata: &FeedMetadata) {
    let current_page = &page;
    let res = get_feed(current_page).await;

    match res {
        Err(why) => panic!("{:?}", why),
        Ok(res) => {
            let items = &res.items;

            match items.len() {
                0 => {
                    let feed_title: &str = &feed_metadata.title;
                    let feed_description: &str = &feed_metadata.description;

                    build_epub(feed_title, feed_description, articles);

                    return;
                }
                _ => {
                    let new_articles = write_article_to_disk(&items);
                    match articles {
                        None => articles = new_articles,
                        Some(_) => articles.as_mut().unwrap().extend(new_articles.unwrap()),
                    }
                }
            }
            let next_page = current_page + 1;
            paginate_feed(next_page, articles, feed_metadata).await;
        }
    }
}

fn build_epub(title: &str, description: &str, articles: Option<Vec<Article>>) {
    let mut builder = EpubBuilder::new(ZipCommand::new().unwrap()).unwrap();
    builder.metadata("title", title).unwrap();
    builder.metadata("title", description).unwrap();
    builder
        .stylesheet(File::open("output/style.css").unwrap())
        .unwrap();
    builder
        .add_cover_image(
            "cover.png",
            File::open("output/cover.png").unwrap(),
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
            EpubContent::new("title.xhtml", File::open("output/title.html").unwrap())
                .title(title)
                .reftype(epub_builder::ReferenceType::TitlePage),
        )
        .unwrap();
    builder.inline_toc();
    builder.set_toc_name(description);
    for article in articles.unwrap() {
        println!("writing article: {:#?}", article);
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
    }
    let curr_dir = env::current_dir().expect("no current directory");
    let temp_file = curr_dir.join("output/mmm.epub");

    let mut file = File::create(temp_file).expect("no file");

    builder.generate(&mut file).expect("no epub");
}

#[tokio::main]
async fn main() {
    let feed_metadata = get_feed_metadata().await.unwrap();
    write_title_page_to_disk(&feed_metadata);

    paginate_feed(1, None, &feed_metadata).await;
}
