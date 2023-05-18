use crate::util;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewsletterData {
    pub title: String,
    pub content: Content,
}

impl NewsletterData {
    pub fn is_title_blank(&self) -> bool {
        util::is_blank(&self.title)
    }

    pub fn is_content_blank(&self) -> bool {
        util::is_blank(&self.content.html) || util::is_blank(&self.content.text)
    }
}

#[derive(Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}