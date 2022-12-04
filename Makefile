.phony:

run:
	cd app && cargo run

env:
	export $(cat .env | xargs)


setup-env:
	cp .env.example .env

setup-db:
	cd app && diesel setup --database-url=$$(cat ../.env | grep DATABASE_URL | cut -d '=' -f 2)

migrate:
	cd app && diesel migration run --database-url=$$(cat ../.env | grep DATABASE_URL | cut -d '=' -f 2)

rollback:
	cd app && diesel migration redo --database-url=$$(cat ../.env| grep DATABASE_URL | cut -d '=' -f 2)

build:
	cd app && cargo build

test:
	cd app && cargo test

binary:
	cd app && ./target/release/samuraicli
