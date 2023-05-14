use serde::Deserialize;

#[derive(Deserialize)]
pub struct BodyData {
    pub title: String,
    pub content: Content,
}

#[derive(Deserialize)]
pub struct Content {
    pub html: String,
    pub text: String,
}
