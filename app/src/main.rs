// write Twitter API 2 get tweets about ekusiadadus using tokio with BEARER_TOKEN
extern crate diesel;

#[macro_use]
mod wrapper;
pub use wrapper::*;

use dotenv::dotenv;

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

    let tweets = app
        .services
        .tweet
        .get_tweets_by_hashtag("ワールドカップ")
        .await
        .unwrap();

    print!("{:?}", tweets);

    app.services.tweet.save_tweets(tweets).await.unwrap();

    let db_tweets = app.services.tweet.search("弾き語り").await.unwrap();

    print!("{:?}", db_tweets);

    Ok(())

    // let client = reqwest::Client::new();
    // let tweet_fileds = "tweet.fields=author_id,created_at,entities,geo,in_reply_to_user_id,lang,possibly_sensitive,referenced_tweets,source,text,withheld";
    // let uri = "https://api.twitter.com/2/tweets/search/recent?query=ekusiadadus".to_string()
    //     + "&"
    //     + tweet_fileds;
    // let response = client
    //     .get(uri)
    //     .bearer_auth(bearer_token)
    //     .send()
    //     .await?
    //     .error_for_status()?;

    // let body = response.text().await?;
    // println!("{}", body);

    // Ok(())
}
