use std::env::current_dir;
use std::fs::File;

use color_eyre::eyre;
// use epub_builder::TocElement;
use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipCommand};

use crate::data::ArticleOnDisk;

use crate::config::*;

pub fn generate_epub(
    title: &str,
    description: &str,
    articles: Vec<ArticleOnDisk>,
) -> Result<(), eyre::Report> {
    let mut builder = EpubBuilder::new(ZipCommand::new()?)?;
    builder.metadata("title", title)?;
    builder.metadata("title", description)?;
    builder.stylesheet(File::open(format!("{}/style.css", &ASSETS_DIR))?)?;
    builder.add_cover_image(
        "cover.png",
        File::open(format!("{}/cover.png", &ASSETS_DIR))?,
        "image/png",
    )?;
    builder.add_content(
        EpubContent::new("cover.xhtml", title.to_string().as_bytes()).reftype(ReferenceType::Cover),
    )?;
    builder.add_content(
        EpubContent::new(
            "title.xhtml",
            File::open(format!("{}/title.html", &OUTPUT_HTML_DIR))?,
        )
        .title(title)
        .reftype(epub_builder::ReferenceType::TitlePage),
    )?;
    builder.inline_toc();
    builder.set_toc_name(description);

    for article in articles {
        builder.add_content(
            EpubContent::new(article.chapter_title, File::open(article.file_path)?)
                .title(article.title)
                .reftype(ReferenceType::Text),
        )?;
        for image in article.images {
            builder.add_resource(image.name, File::open(image.path)?, image.mime_type)?;
        }
    }
    let curr_dir = current_dir().expect("no current directory");
    let temp_file = curr_dir.join(format!("{}/{}", &OUTPUT_DIR, &EPUB_NAME));

    let mut file = File::create(temp_file).expect("no file");

    builder.generate(&mut file)
}
