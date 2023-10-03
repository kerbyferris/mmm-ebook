use chrono::DateTime;
use chrono::FixedOffset;

#[derive(Debug)]
pub struct Image {
    pub name: String,
    pub path: String,
    pub mime_type: String,
}

#[derive(Debug)]
pub struct ArticleOnDisk {
    pub title: String,
    pub file_path: String,
    pub chapter_title: String,
    pub images: Vec<Image>,
}

#[derive(Debug)]
pub struct Article {
    pub title: String,
    pub date: String,
    pub timestamp: DateTime<FixedOffset>,
    pub content: String,
}

#[derive(Debug)]
pub struct FeedMetadata {
    pub title: String,
    pub description: String,
}
