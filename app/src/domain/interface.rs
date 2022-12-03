use crate::domain::model::*;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ITweetRepository {
    async fn find_by_id(&self, id: &TweetID) -> Result<Tweet>;
    async fn save(&self, tweet: Tweet) -> Result<()>;
    async fn save_tweets(&self, tweets: Vec<Tweet>) -> Result<()>;
    async fn search(&self, query: &str) -> Result<Vec<Tweet>>;
    async fn get_tweets_by_hashtag(&self, hashtag: &str) -> Result<Vec<Tweet>>;
    async fn delete(&self, id: &TweetID) -> Result<()>;
    async fn delete_tweet(&self, id: &TweetID) -> Result<()>;
    async fn favorite_tweet(&self, id: &TweetID) -> Result<()>;
}

#[async_trait]
pub trait IHttpClient {
    async fn get(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response>;
    async fn post(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
        body: Option<String>,
    ) -> Result<reqwest::Response>;
    async fn put(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
        body: String,
    ) -> Result<reqwest::Response>;
    async fn delete(
        &self,
        url: &str,
        header: Option<reqwest::header::HeaderMap>,
    ) -> Result<reqwest::Response>;
}
