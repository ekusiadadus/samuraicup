use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Default, Eq, Hash)]
pub struct TweetID(pub String);

// TweetID to String
impl From<TweetID> for String {
    fn from(tweet_id: TweetID) -> Self {
        tweet_id.0
    }
}

// String to TweetID
impl From<String> for TweetID {
    fn from(tweet_id: String) -> Self {
        TweetID(tweet_id)
    }
}
