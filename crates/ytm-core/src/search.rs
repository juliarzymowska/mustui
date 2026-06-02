use crate::{models::SearchResults, ytdlp, Backend, Result};

impl Backend {
    pub async fn search(&self, query: &str) -> Result<SearchResults> {
        ytdlp::search(query, 10).await
    }
}

#[cfg(test)]
mod tests {
    use crate::Backend;

    #[tokio::test]
    #[ignore = "requires yt-dlp + network"]
    async fn search_returns_tracks() {
        let backend = Backend::new().await.expect("backend init");
        let results = backend.search("lofi hip hop").await.expect("search");
        assert!(!results.tracks.is_empty());
        let first = &results.tracks[0];
        assert!(!first.id.0.is_empty());
        assert!(!first.title.is_empty());
    }
}
