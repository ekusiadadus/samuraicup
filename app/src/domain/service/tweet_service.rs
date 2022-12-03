use crate::domain::interface::*;
use crate::domain::model::*;
use crate::error::*;
use std::sync::Arc;

#[derive(Clone)]

pub struct TweetService {
    tweet_repo: Arc<dyn ITweetRepository + Send + Sync>,
}

// pub struct FindTweetOutput {
//     data: Vec<Tweet>,
// }

impl TweetService {
    pub fn new(tweet_repo: Arc<dyn ITweetRepository + Send + Sync>) -> Self {
        Self { tweet_repo }
    }

    // pub async fn create(
    //     &self,
    //     id: String,
    //     text: String,
    //     author_id: String,
    //     created_at: String,
    //     entities: String,
    //     geo: Option<String>,
    //     in_reply_to_user_id: Option<String>,
    //     lang: String,
    //     possibly_sensitive: Option<bool>,
    //     referenced_tweets: Option<String>,
    //     source: String,
    //     withheld: Option<String>,
    // ) -> Result<()> {
    //     let tweet = Tweet::new(
    //         id,
    //         text,
    //         author_id,
    //         created_at,
    //         entities,
    //         geo,
    //         in_reply_to_user_id,
    //         lang,
    //         possibly_sensitive,
    //         referenced_tweets,
    //         source,
    //         withheld,
    //     );
    //     self.tweet_repo.save(tweet.clone()).await?;
    //     Ok(())
    // }

    // pub async fn find_by_id(&self, id: &TweetID) -> Result<FindTweetOutput> {
    //     let tweet = self.tweet_repo.find_by_id(id).await?;
    //     Ok(FindTweetOutput { data: vec![tweet] })
    // }

    pub async fn search(&self, query: &str) -> Result<Vec<Tweet>> {
        let tweets = self.tweet_repo.search(query).await?;
        Ok(tweets)
    }

    pub async fn get_tweets_by_hashtag(&self, hashtag: &str) -> Result<Vec<Tweet>> {
        let tweets = self.tweet_repo.get_tweets_by_hashtag(hashtag).await?;
        Ok(tweets)
    }

    // pub async fn save(&self, tweet: Tweet) -> Result<()> {
    //     self.tweet_repo.save(tweet).await?;
    //     Ok(())
    // }

    pub async fn save_tweets(&self, tweets: Vec<Tweet>) -> Result<()> {
        self.tweet_repo.save_tweets(tweets).await?;
        Ok(())
    }
}
