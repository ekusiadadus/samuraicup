use serde::*;

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct Tweet {
    pub id: String,
    pub text: String,
    pub author_id: String,
    pub created_at: String,
    pub entities: Option<serde_json::Value>,
    pub geo: Option<serde_json::Value>,
    pub in_reply_to_user_id: Option<String>,
    pub lang: Option<String>,
    pub possibly_sensitive: Option<bool>,
    pub referenced_tweets: Option<Vec<serde_json::Value>>,
    pub source: Option<String>,
    pub withheld: Option<serde_json::Value>,
}

impl Tweet {
    pub fn new(
        id: String,
        text: String,
        author_id: String,
        created_at: String,
        entities: Option<serde_json::Value>,
        geo: Option<serde_json::Value>,
        in_reply_to_user_id: Option<String>,
        lang: Option<String>,
        possibly_sensitive: Option<bool>,
        referenced_tweets: Option<Vec<serde_json::Value>>,
        source: Option<String>,
        withheld: Option<serde_json::Value>,
    ) -> Self {
        Tweet {
            id,
            text,
            author_id,
            created_at,
            entities,
            geo,
            in_reply_to_user_id,
            lang,
            possibly_sensitive,
            referenced_tweets,
            source,
            withheld,
        }
    }
}
