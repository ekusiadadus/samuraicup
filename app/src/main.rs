// write Twitter API 2 get tweets about ekusiadadus using tokio with BEARER_TOKEN
extern crate diesel;

#[macro_use]
mod wrapper;
use serde::__private::de::IdentifierDeserializer;
pub use wrapper::*;

use dotenv::dotenv;

use crate::domain::model::TweetID;

mod domain;
mod infra;
mod initializer;
mod repository;
mod schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool_size = std::env::var("DATABASE_POOL_SIZE")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(5);
    let bearer_token = std::env::var("BEARER_TOKEN").expect("BEARER_TOKEN not set");
    // let tweets_table_name = std::env::var("TWEETS_TABLE_NAME").expect("TWEETS_TABLE_NAME not set");

    let app = initializer::new(initializer::Config {
        db_url: db_url,
        db_pool_size: db_pool_size,
        // tweets_table_name: tweets_table_name,
        bearer_token: bearer_token,
    })
    .await;

    app.infras
        .ensure_initialized()
        .await
        .expect("Infra initialization error");

    // let tweets = app
    //     .services
    //     .tweet
    //     .get_tweets_by_hashtag("ワールドカップ")
    //     .await
    //     .unwrap();

    // print!("{:?}", tweets);

    // app.services.tweet.save_tweets(tweets).await.unwrap();

    // get tweets by every 1 minute and save to db
    // tweet view
    loop {
        let latest_tweet = app.services.tweet.get_latest_tweets(1).await.unwrap();

        let latest_tweet_id = if latest_tweet.len() == 0 {
            TweetID("0".to_string())
        } else {
            TweetID(latest_tweet[0].id.clone())
        };

        let tweets = if latest_tweet.len() == 0 {
            app.services
                .tweet
                .get_tweets("ワールドカップ")
                .await
                .unwrap()
        } else {
            app.services
                .tweet
                .get_tweets_after_id("ワールドカップ", &latest_tweet_id)
                .await
                .unwrap()
        };

        app.services
            .tweet
            .save_tweets(tweets.clone())
            .await
            .unwrap();

        for tweet in tweets {
            // author
            println!("author: {}", tweet.author_id);
            // tweet
            println!("{}", tweet.text);
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
