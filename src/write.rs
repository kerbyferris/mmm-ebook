use crate::config::OUTPUT_HTML_DIR;
use crate::config::OUTPUT_IMG_DIR;

use crate::data::Article;
use crate::data::ArticleOnDisk;
use crate::data::FeedMetadata;
use crate::data::Image;

use chrono::Datelike;
use color_eyre::eyre;

use std::fs;
use std::fs::File;
use std::io::Write;

use build_html::*;

use lol_html::{element, rewrite_str, RewriteStrSettings};

use std::io::copy;

fn get_image_mime_type(image_name: &str) -> Result<String, eyre::Report> {
    let file_ext_fallback = "jpg";
    let mime_type_fallback = "image/jpg".to_string();

    let &file_ext = image_name
        .split('.')
        .collect::<Vec<&str>>()
        .last()
        .unwrap_or(&file_ext_fallback);

    match file_ext {
        "jpg" | "JPG" | "jpeg" | "JPEG" => Ok(mime_type_fallback),
        "png" | "PNG" => Ok("image/png".to_string()),
        &_ => Ok(mime_type_fallback),
    }
}

pub fn title_page_to_disk(feed_metadata: &FeedMetadata) -> Result<&FeedMetadata, eyre::Report> {
    let page = build_html::HtmlPage::new()
        .with_header(1, &feed_metadata.title)
        .with_paragraph(&feed_metadata.description)
        .to_html_string();

    let file_path = format!("{}/title.html", &OUTPUT_HTML_DIR);
    let mut file = fs::File::create(file_path)?;
    file.write_all(page.as_bytes())?;

    Ok(feed_metadata)
}

pub fn article_to_disk(article: &Article) -> Result<Option<ArticleOnDisk>, eyre::Report> {
    let get_images = false; // TODO

    let timestamp = article.timestamp;

    let year = timestamp.year().to_string();

    let date_path_string = timestamp.format("%Y%m%d").to_string();

    let mut content = article.content.to_string();
    let mut images = vec![];

    if get_images {
        (content, images) = update_img_html(content.to_string(), &date_path_string, &year)?;
    }

    let title = &article.title;

    let page = build_html::HtmlPage::new()
        .with_paragraph(&article.date)
        .with_header(1, title)
        .with_paragraph(content)
        .to_html_string();

    let output_dir = format!("{}/{}", &OUTPUT_HTML_DIR, &year);

    let chapter_title = format!("chapter_{}.html", &date_path_string);

    let file_path = format!("{}/{}", &output_dir, &chapter_title);

    let mut file = File::create(&file_path)?;
    file.write_all(page.as_bytes())?;

    let article_on_disk = ArticleOnDisk {
        title: title.to_string(),
        file_path,
        chapter_title,
        images,
    };
    Ok(Some(article_on_disk))
}

fn update_img_html(
    content: String,
    date_path_string: &String,
    year: &String,
) -> Result<(String, Vec<Image>), eyre::Report> {
    let mut article_images: Vec<Image> = vec![];
    let element_content_handlers = vec![element!("img[src]", |el| {
        let image_url = el.get_attribute("src").unwrap();

        let output_dir = format!("{}/{}", &OUTPUT_HTML_DIR, &year);
        fs::create_dir_all(output_dir)?;

        let image_url_fragments: Vec<&str> = image_url.split('/').collect();
        let image_name = image_url_fragments.last().unwrap();
        let unique_image_name = format!("{}-{}", date_path_string, image_name);

        let img_file_path = format!("{}/{}", &OUTPUT_IMG_DIR, unique_image_name);
        let mut file = fs::File::create(&img_file_path)?;

        let image_meta = Image {
            name: unique_image_name.to_string(),
            path: img_file_path,
            mime_type: get_image_mime_type(image_name)?,
        };

        article_images.push(image_meta);

        let mut image_result = reqwest::blocking::get(&image_url)?;
        let _ = copy(&mut image_result, &mut file);

        el.set_attribute("src", &unique_image_name)?;

        Ok(())
    })];
    let output = rewrite_str(
        &content,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )?;

    Ok((output, article_images))
}
