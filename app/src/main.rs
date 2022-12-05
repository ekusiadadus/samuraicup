// write Twitter API 2 get tweets about ekusiadadus using tokio with BEARER_TOKEN
extern crate diesel;

#[macro_use]
mod wrapper;
pub use wrapper::*;

use dotenv::dotenv;
use owo_colors::{AnsiColors, OwoColorize};
use rand::Rng;
use std::ffi::OsString;

use clap::{ColorChoice, Command};

use crate::domain::model::TweetID;

mod domain;
mod infra;
mod initializer;
mod repository;
mod schema;

fn cli() -> Command {
    Command::new("samuraicup")
        .about("🌸 World Cup 2022 CLI for Japanese football fans 🌸")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        // real: color red
        .subcommand(Command::new("real").about("⚽ワールドカップをリアルタイムで確認する"))
        .subcommand(Command::new("search").about("🥅ワールドカップのツイートを取得する"))
        .subcommand(Command::new("keisuke").about("📣本田圭佑の動向を取得する"))
}

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

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("real", sub_matches)) => {
            let mut rng = rand::thread_rng();
            let color = owo_colors::Rgb(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );

            let text_color = owo_colors::Rgb(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );
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
                    println!(
                        "{} {}",
                        format!("{}", tweet.author_id).color(color).bold(),
                        format!("{}", tweet.text).color(text_color),
                    );
                }

                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
        Some(("search", sub_matches)) => {
            let tweets = app
                .services
                .tweet
                .get_tweets("ワールドカップ")
                .await
                .unwrap();

            for tweet in tweets {
                println!("{}", tweet.text);
            }
        }
        Some(("keisuke", sub_matches)) => {
            let mut rng = rand::thread_rng();

            let color = owo_colors::Rgb(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );

            let text_color = owo_colors::Rgb(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );
            let tweets = app.services.tweet.get_tweets("本田圭佑").await.unwrap();
            let debug_str = "    ";
            for tweet in tweets {
                println!(
                    "{}",
                    format!(
                        "{}{}{}",
                        debug_str,
                        format!("{}: ", tweet.author_id).color(color),
                        tweet.text.color(text_color)
                    )
                );
            }
        }
        Some((ext, sub_matches)) => {
            let args = sub_matches
                .get_many::<OsString>("")
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            println!("Calling out to {:?} with {:?}", ext, args);
        }
        _ => unreachable!(),
    }

    Ok(())
}
