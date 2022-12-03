-- Your SQL goes here
CREATE TABLE tweet_records (
    id VARCHAR(255) NOT NULL,
    text VARCHAR(255) NOT NULL,
    author_id VARCHAR(255) NOT NULL,
    created_at VARCHAR(255) NOT NULL,
    entities VARCHAR(255) NOT NULL,
    geo VARCHAR(255) NULL,
    in_reply_to_user_id VARCHAR(255) NULL,
    lang VARCHAR(255) NOT NULL,
    possibly_sensitive BOOLEAN NULL,
    referenced_tweets VARCHAR(255) NULL,
    source VARCHAR(255) NOT NULL,
    withheld VARCHAR(255) NULL,
    PRIMARY KEY (id)
);
