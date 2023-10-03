use crate::config::OUTPUT_HTML_DIR;
use crate::config::OUTPUT_IMG_DIR;

use crate::data::Article;
use crate::data::FeedMetadata;
use crate::data::Image;

use chrono::DateTime;
use chrono::Datelike;

use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;

use build_html::*;

use lol_html::{element, rewrite_str, RewriteStrSettings};

use std::io::copy;

fn get_mime_type(image_name: &str) -> Result<String, Box<dyn Error>> {
    let fragments: Vec<&str> = image_name.split('.').collect();
    let suffix = fragments.last();

    match suffix {
        // TODO: match jpg JPG jpeg JPEG png PNG
        None => println!("none"),
        Some(s) => println!("{}", s),
    }
    Ok("image/jpg".to_string())
}

pub fn title_page_to_disk(feed_metadata: &FeedMetadata) -> Result<&FeedMetadata, Box<dyn Error>> {
    let page = build_html::HtmlPage::new()
        .with_header(1, &feed_metadata.title)
        .with_paragraph(&feed_metadata.description)
        .to_html_string();

    let file_path = format!("{}/title.html", &OUTPUT_HTML_DIR);
    let mut file = fs::File::create(file_path)?;
    file.write_all(page.as_bytes())?;

    Ok(feed_metadata)
}

pub fn article_to_disk(items: &Vec<rss::Item>) -> Option<Vec<Article>> {
    let mut articles: Vec<Article> = vec![];
    for item in items {
        let date = DateTime::parse_from_rfc2822(item.pub_date.as_ref().unwrap()).unwrap();

        let year = date.year().to_string();

        let date_string = date.format("%b %d, %Y").to_string();
        let date_path_string = date.format("%Y%m%d").to_string();

        let content = item.content.as_ref().unwrap();
        let title = item.title.as_ref().unwrap();

        let (new_content, images) =
            update_img_html(content.to_string(), &date_path_string, &year).unwrap();

        let page = build_html::HtmlPage::new()
            .with_paragraph(&date_string)
            .with_header(1, title)
            .with_paragraph(new_content)
            .to_html_string();

        let output_dir = format!("{}/{}", &OUTPUT_HTML_DIR, &year);

        let chapter_title = format!("chapter_{}.html", &date_path_string);

        let file_path = format!("{}/{}", &output_dir, &chapter_title);

        let mut file = File::create(&file_path).unwrap();
        file.write_all(page.as_bytes()).unwrap();

        let article = Article {
            title: title.to_string(),
            file_path,
            chapter_title,
            images,
        };

        println!("{:#?}", article);
        articles.push(article)
    }
    Some(articles)
}

fn update_img_html(
    content: String,
    date_path_string: &String,
    year: &String,
) -> Result<(String, Vec<Image>), Box<dyn Error>> {
    let mut article_images: Vec<Image> = vec![];
    let element_content_handlers = vec![element!("img[src]", |el| {
        let image_url = el.get_attribute("src").unwrap();

        let output_dir = format!("{}/{}", &OUTPUT_HTML_DIR, &year);
        fs::create_dir_all(output_dir)?;

        let image_result = reqwest::blocking::get(&image_url);

        match image_result {
            Err(e) => println!("{}", e),
            Ok(mut image) => {
                let image_url_fragments: Vec<&str> = image_url.split('/').collect();
                let image_name = image_url_fragments.last().unwrap();
                let unique_image_name = format!("{}-{}", date_path_string, image_name);

                let img_file_path = format!("{}/{}", &OUTPUT_IMG_DIR, unique_image_name);
                let mut file = fs::File::create(&img_file_path)?;

                let image_meta = Image {
                    name: unique_image_name.to_string(),
                    path: img_file_path,
                    mime_type: get_mime_type(image_name).unwrap(),
                };

                article_images.push(image_meta);

                copy(&mut image, &mut file).unwrap();

                el.set_attribute("src", &unique_image_name).unwrap();
            }
        }

        Ok(())
    })];
    let output = rewrite_str(
        &content,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )
    .unwrap();

    Ok((output, article_images))
}
