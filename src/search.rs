use crate::{client::Backend, error::Result, models::SearchResults, ytdlp};

impl Backend {
    pub async fn search(&self, query: &str) -> Result<SearchResults> {
        ytdlp::search(query, 10).await
    }
}
