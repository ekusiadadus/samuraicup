use crate::domain::interface::*;
use crate::domain::model::*;
use crate::error::*;
use crate::infra::DBConnector;
use crate::schema::tweet_records;
use async_trait::async_trait;
use diesel::dsl::*;
use diesel::prelude::*;
use serde::*;
use std::sync::Arc;

#[derive(Debug)]
pub enum TweetRepoError {
    HttpClientError,
}

impl IServiceError for TweetRepoError {
    fn error_type(&self) -> String {
        use TweetRepoError::*;
        match self {
            HttpClientError => "http_client_error",
        }
        .to_string()
    }

    fn status_code(&self) -> http::StatusCode {
        use TweetRepoError::*;
        match self {
            HttpClientError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Queryable, Insertable, Identifiable)]
pub struct TweetRecord {
    id: String,
    text: String,
    author_id: String,
    created_at: String,
    entities: String,
    geo: Option<String>,
    in_reply_to_user_id: Option<String>,
    lang: String,
    possibly_sensitive: Option<bool>,
    referenced_tweets: Option<String>,
    source: String,
    withheld: Option<String>,
    bigquery: bool,
}

impl TweetRecord {
    pub fn to_model(self) -> Result<Tweet> {
        let entities: Option<serde_json::Value> = serde_json::from_str(&self.entities).unwrap();
        let geo: Option<serde_json::Value> = match self.geo {
            Some(geo) => Some(serde_json::from_str(&geo).unwrap()),
            None => None,
        };
        let referenced_tweets: Option<Vec<serde_json::Value>> = match self.referenced_tweets {
            Some(referenced_tweets) => Some(serde_json::from_str(&referenced_tweets).unwrap()),
            None => None,
        };
        let withheld: Option<serde_json::Value> = match self.withheld {
            Some(withheld) => Some(serde_json::from_str(&withheld).unwrap()),
            None => None,
        };
        Ok(Tweet::new(
            self.id,
            self.text,
            self.author_id,
            self.created_at,
            entities,
            geo,
            self.in_reply_to_user_id,
            Some(self.lang),
            self.possibly_sensitive,
            referenced_tweets,
            Some(self.source),
            withheld,
        ))
    }

    pub fn from_model(tweet: Tweet) -> Result<Self> {
        let entities = serde_json::to_string(&tweet.entities).unwrap();
        let geo = match tweet.geo {
            Some(geo) => Some(serde_json::to_string(&geo).unwrap()),
            None => None,
        };
        let referenced_tweets = match tweet.referenced_tweets {
            Some(referenced_tweets) => Some(serde_json::to_string(&referenced_tweets).unwrap()),
            None => None,
        };
        let withheld = match tweet.withheld {
            Some(withheld) => Some(serde_json::to_string(&withheld).unwrap()),
            None => None,
        };
        Ok(TweetRecord {
            id: tweet.id,
            text: tweet.text,
            author_id: tweet.author_id,
            created_at: tweet.created_at,
            entities,
            geo,
            in_reply_to_user_id: tweet.in_reply_to_user_id,
            lang: tweet.lang.unwrap(),
            possibly_sensitive: tweet.possibly_sensitive,
            referenced_tweets,
            source: tweet.source.unwrap(),
            withheld,
            bigquery: false,
        })
    }
}

pub struct TweetRepository {
    db: DBConnector,
    http_client: Arc<dyn IHttpClient + Sync + Send>,
    bearer_token: String,
}

impl TweetRepository {
    pub fn new(
        db: DBConnector,
        http_client: Arc<dyn IHttpClient + Sync + Send>,
        bearer_token: String,
    ) -> Self {
        Self {
            db,
            http_client,
            bearer_token,
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Default)]

pub struct TweetResponse {
    data: Vec<Tweet>,
    meta: Option<TweetResponseMeta>,
}

#[derive(Clone, Deserialize, Serialize, Default)]
pub struct TweetResponseMeta {
    pub newest_id: String,
    pub oldest_id: String,
    pub result_count: i64,
    pub next_token: Option<String>,
}

#[async_trait]
impl ITweetRepository for TweetRepository {
    async fn find_by_id(&self, id: &TweetID) -> Result<Tweet> {
        let record = self
            .db
            .first::<TweetRecord, _>(
                tweet_records::table.filter(tweet_records::id.eq(id.0.clone())),
            )
            .await?;
        record.to_model()
    }

    async fn search(&self, query: &str) -> Result<Vec<Tweet>> {
        let records = self
            .db
            .load::<TweetRecord, _>(
                tweet_records::table.filter(tweet_records::text.like(format!("%{}%", query))),
            )
            .await?;
        records
            .into_iter()
            .map(|record| record.to_model())
            .collect::<Result<Vec<Tweet>>>()
    }

    async fn get_tweets_by_hashtag(&self, hashtag: &str) -> Result<Vec<Tweet>> {
        // remove retweets
        let tweet_fileds = "tweet.fields=author_id,created_at,entities,geo,in_reply_to_user_id,lang,possibly_sensitive,referenced_tweets,source,text,withheld&max_results=10";
        let uri = "https://api.twitter.com/2/tweets/search/recent?query=ekusiadadus -is: retweet"
            .to_string()
            + "&"
            + tweet_fileds;
        let uri = uri.replace("ekusiadadus", hashtag);
        let mut headers = reqwest::header::HeaderMap::new();
        // add bearer_token
        let bearer_token = format!("Bearer {}", self.bearer_token);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            bearer_token.parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        let response = self.http_client.get(&uri, Some(headers)).await.unwrap();

        let body = response.text().await.unwrap();

        print!("{}", body);

        let tweets = serde_json::from_str::<TweetResponse>(&body).unwrap();

        print!("{:?}", tweets.data);

        Ok(tweets.data)
    }

    async fn save(&self, tweet: Tweet) -> Result<()> {
        let record = TweetRecord::from_model(tweet)?;
        self.db
            .execute(replace_into(tweet_records::table).values::<TweetRecord>(record))
            .await?;
        Ok(())
    }

    async fn save_tweets(&self, tweets: Vec<Tweet>) -> Result<()> {
        let records = tweets
            .into_iter()
            .map(|tweet| TweetRecord::from_model(tweet))
            .collect::<Result<Vec<TweetRecord>>>()?;
        for record in records {
            self.db
                .execute(replace_into(tweet_records::table).values::<TweetRecord>(record))
                .await?;
        }
        Ok(())
    }

    async fn delete(&self, id: &TweetID) -> Result<()> {
        self.db
            .execute(delete(tweet_records::table).filter(tweet_records::id.eq(id.0.clone())))
            .await?;
        Ok(())
    }

    async fn delete_tweet(&self, id: &TweetID) -> Result<()> {
        // delete tweet from twitter
        let uri = format!("https://api.twitter.com/2/tweets/{}", id.0);
        let mut headers = reqwest::header::HeaderMap::new();
        // add bearer_token

        let bearer_token = format!(
            "
        Bearer {}",
            self.bearer_token
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            bearer_token.parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );
        let response = self.http_client.delete(&uri, Some(headers)).await.unwrap();

        let body = response.text().await.unwrap();

        print!("{}", body);

        Ok(())
    }

    async fn favorite_tweet(&self, id: &TweetID) -> Result<()> {
        // favorite tweet from twitter
        let uri = format!("https://api.twitter.com/2/tweets/{}/like", id.0);
        let mut headers = reqwest::header::HeaderMap::new();
        // add bearer_token

        let bearer_token = format!(
            "
        Bearer {}",
            self.bearer_token
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            bearer_token.parse().unwrap(),
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        let response = self
            .http_client
            .post(&uri, Some(headers), None)
            .await
            .unwrap();

        let body = response.text().await.unwrap();

        print!("{}", body);

        Ok(())
    }
}

// TweetResponse
// {
//   "data": [
//     {
//       "possibly_sensitive": false,
//       "referenced_tweets": [
//         {
//           "type": "quoted",
//           "id": "1592094998769434627"
//         }
//       ],
//       "id": "1592104440001359873",
//       "entities": {
//         "urls": [
//           {
//             "start": 25,
//             "end": 48,
//             "url": "https://t.co/fYEKtksj9O",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1592104440001359873/photo/1",
//             "display_url": "pic.twitter.com/fYEKtksj9O",
//             "media_key": "3_1592104369092460544"
//           },
//           {
//             "start": 49,
//             "end": 72,
//             "url": "https://t.co/luAE3sLVVu",
//             "expanded_url": "https://twitter.com/Scenify_jp/status/1592094998769434627",
//             "display_url": "twitter.com/Scenify_jp/sta…"
//           }
//         ]
//       },
//       "created_at": "2022-11-14T10:37:33.000Z",
//       "edit_history_tweet_ids": [
//         "1592104440001359873"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter Web App",
//       "lang": "ja",
//       "text": "完全に、crypkoで草\nサービスとしてはよさげ https://t.co/fYEKtksj9O https://t.co/luAE3sLVVu"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1592094492223737856",
//       "entities": {
//         "urls": [
//           {
//             "start": 14,
//             "end": 37,
//             "url": "https://t.co/jcNCxgn7bG",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1592094492223737856/photo/1",
//             "display_url": "pic.twitter.com/jcNCxgn7bG",
//             "media_key": "3_1592094450511097857"
//           }
//         ]
//       },
//       "created_at": "2022-11-14T09:58:01.000Z",
//       "edit_history_tweet_ids": [
//         "1592094492223737856"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter Web App",
//       "lang": "ja",
//       "text": "これ、面白すぎるだろ（笑） https://t.co/jcNCxgn7bG"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1592078474898214912",
//       "entities": {
//         "urls": [
//           {
//             "start": 18,
//             "end": 41,
//             "url": "https://t.co/zcYrCUoQwI",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1592078474898214912/photo/1",
//             "display_url": "pic.twitter.com/zcYrCUoQwI",
//             "media_key": "3_1592078471530164225"
//           }
//         ]
//       },
//       "created_at": "2022-11-14T08:54:22.000Z",
//       "edit_history_tweet_ids": [
//         "1592078474898214912"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter for Android",
//       "lang": "ja",
//       "text": "うまうま\n\n油淋鶏が、一番好きかも https://t.co/zcYrCUoQwI"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591867817766576128",
//       "entities": {
//         "urls": [
//           {
//             "start": 42,
//             "end": 65,
//             "url": "https://t.co/zBi4OawaL9",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591867817766576128/photo/1",
//             "display_url": "pic.twitter.com/zBi4OawaL9",
//             "media_key": "3_1591867811827441665"
//           },
//           {
//             "start": 42,
//             "end": 65,
//             "url": "https://t.co/zBi4OawaL9",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591867817766576128/photo/1",
//             "display_url": "pic.twitter.com/zBi4OawaL9",
//             "media_key": "3_1591867815220645890"
//           }
//         ]
//       },
//       "created_at": "2022-11-13T18:57:17.000Z",
//       "edit_history_tweet_ids": [
//         "1591867817766576128"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter for Android",
//       "lang": "ja",
//       "text": "鹿肉のパイ生地包は、美味しかったんだけど\n\nサラダについてくるプリンが、苦手だった https://t.co/zBi4OawaL9"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591853814529003521",
//       "entities": {
//         "urls": [
//           {
//             "start": 0,
//             "end": 23,
//             "url": "https://t.co/29rdnXshA3",
//             "expanded_url": "https://gist.github.com/ekusiadadus/e267f700ff76847ba22cd280b0940882",
//             "display_url": "gist.github.com/ekusiadadus/e2…",
//             "images": [
//               {
//                 "url": "https://pbs.twimg.com/news_img/1591853879817560066/1uWb-I1f?format=png&name=orig",
//                 "width": 1280,
//                 "height": 640
//               },
//               {
//                 "url": "https://pbs.twimg.com/news_img/1591853879817560066/1uWb-I1f?format=png&name=150x150",
//                 "width": 150,
//                 "height": 150
//               }
//             ],
//             "status": 200,
//             "title": "get_tweets.rs",
//             "description": "get_tweets.rs. GitHub Gist: instantly share code, notes, and snippets.",
//             "unwound_url": "https://gist.github.com/ekusiadadus/e267f700ff76847ba22cd280b0940882"
//           }
//         ]
//       },
//       "created_at": "2022-11-13T18:01:39.000Z",
//       "edit_history_tweet_ids": [
//         "1591853814529003521"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter Web App",
//       "lang": "zxx",
//       "text": "https://t.co/29rdnXshA3"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591840639636865026",
//       "entities": {
//         "urls": [
//           {
//             "start": 82,
//             "end": 105,
//             "url": "https://t.co/XcTHXGMXKc",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591840639636865026/photo/1",
//             "display_url": "pic.twitter.com/XcTHXGMXKc",
//             "media_key": "3_1591840607982350337"
//           }
//         ]
//       },
//       "created_at": "2022-11-13T17:09:18.000Z",
//       "edit_history_tweet_ids": [
//         "1591840639636865026"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter Web App",
//       "lang": "ja",
//       "text": "Rust Diesel Sqlite に multi thread で書き込もうとしている人を、この世に他に一人だけ見つけ出した。\n\nまじで、何やっているんだろう https://t.co/XcTHXGMXKc"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591674286527610882",
//       "entities": {
//         "urls": [
//           {
//             "start": 25,
//             "end": 48,
//             "url": "https://t.co/PLoz3sOoe4",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591674286527610882/photo/1",
//             "display_url": "pic.twitter.com/PLoz3sOoe4",
//             "media_key": "3_1591674226905186307"
//           }
//         ]
//       },
//       "created_at": "2022-11-13T06:08:16.000Z",
//       "edit_history_tweet_ids": [
//         "1591674286527610882"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter Web App",
//       "lang": "ja",
//       "text": "Pythonを書いていなさすぎるという強い危機感 https://t.co/PLoz3sOoe4"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591665072564473856",
//       "entities": {
//         "urls": [
//           {
//             "start": 115,
//             "end": 138,
//             "url": "https://t.co/PhUTV2Wfhg",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591665072564473856/photo/1",
//             "display_url": "pic.twitter.com/PhUTV2Wfhg",
//             "media_key": "3_1591665055640080384"
//           }
//         ]
//       },
//       "created_at": "2022-11-13T05:31:39.000Z",
//       "edit_history_tweet_ids": [
//         "1591665072564473856"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter Web App",
//       "lang": "ja",
//       "text": "切れそう\n\nexpected &amp;mut SqliteConnection, found &amp;PooledConnection&lt;ConnectionManager&lt;SqliteConnection&gt; https://t.co/PhUTV2Wfhg"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591562335638163456",
//       "entities": {
//         "urls": [
//           {
//             "start": 9,
//             "end": 32,
//             "url": "https://t.co/4wPIr2sniI",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591562335638163456/photo/1",
//             "display_url": "pic.twitter.com/4wPIr2sniI",
//             "media_key": "3_1591562331414462464"
//           }
//         ]
//       },
//       "created_at": "2022-11-12T22:43:25.000Z",
//       "edit_history_tweet_ids": [
//         "1591562335638163456"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter for Android",
//       "lang": "ja",
//       "text": "朝ですぜ\n\n寝る https://t.co/4wPIr2sniI"
//     },
//     {
//       "possibly_sensitive": false,
//       "id": "1591560289069432832",
//       "entities": {
//         "urls": [
//           {
//             "start": 4,
//             "end": 27,
//             "url": "https://t.co/VCpgo9f8U7",
//             "expanded_url": "https://twitter.com/ekusiadadus/status/1591560289069432832/photo/1",
//             "display_url": "pic.twitter.com/VCpgo9f8U7",
//             "media_key": "3_1591560285512663040"
//           }
//         ]
//       },
//       "created_at": "2022-11-12T22:35:17.000Z",
//       "edit_history_tweet_ids": [
//         "1591560289069432832"
//       ],
//       "author_id": "1299285051436273664",
//       "source": "Twitter for Android",
//       "lang": "ja",
//       "text": "綺麗や https://t.co/VCpgo9f8U7"
//     }
//   ],
//   "meta": {
//     "newest_id": "1592104440001359873",
//     "oldest_id": "1591560289069432832",
//     "result_count": 10,
//     "next_token": "b26v89c19zqg8o3fpzhjn2uq157wnmzphblk5a42ls1a5"
//   }
// }
