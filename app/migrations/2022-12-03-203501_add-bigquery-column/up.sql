-- Your SQL goes here
ALTER TABLE tweet_records
  ADD bigquery BOOLEAN DEFAULT FALSE NOT NULL;
