use crate::utils;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewsletterData {
    pub title: String,
    pub text_content: String,
    pub html_content: String,
    pub idempotency_key: String,
}

impl NewsletterData {
    pub fn is_title_blank(&self) -> bool {
        utils::is_blank(&self.title)
    }

    pub fn is_html_blank(&self) -> bool {
        utils::is_blank(&self.html_content)
    }

    pub fn is_text_blank(&self) -> bool {
        utils::is_blank(&self.text_content)
    }
}
