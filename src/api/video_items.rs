use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VideoItems {
    pub items: Vec<VideoItem>
}

#[derive(Debug, Deserialize)]
pub struct VideoItem {
    pub id: String,
    #[serde(rename(deserialize = "contentDetails"))]
    pub content_details: ContentDetails
}

#[derive(Debug, Deserialize)]
pub struct ContentDetails {
    pub duration: String
}