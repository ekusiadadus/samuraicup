// @generated automatically by Diesel CLI.

diesel::table! {
    tweet_records (id) {
        id -> Text,
        text -> Text,
        author_id -> Text,
        created_at -> Text,
        entities -> Text,
        geo -> Nullable<Text>,
        in_reply_to_user_id -> Nullable<Text>,
        lang -> Text,
        possibly_sensitive -> Nullable<Bool>,
        referenced_tweets -> Nullable<Text>,
        source -> Text,
        withheld -> Nullable<Text>,
        bigquery -> Bool,
    }
}
