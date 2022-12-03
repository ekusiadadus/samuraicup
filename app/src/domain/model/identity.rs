use std::hash::Hash;

#[derive(Clone, Debug, PartialEq, Default, Eq, Hash)]
pub struct TweetID(pub String);
