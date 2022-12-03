.phony:

run:
	cd app && cargo run

env:
	export $$(cat .env | grep -v ^#)

setup-env:
	cp .env.example .env

setup-db:
	diesel setup --database-url=$$(cat .env | grep DATABASE_URL | cut -d '=' -f 2)

migrate:
	diesel migration run --database-url=$$(cat .env | grep DATABASE_URL | cut -d '=' -f 2)

rollback:
	diesel migration redo --database-url=$$(cat .env | grep DATABASE_URL | cut -d '=' -f 2)

build:
	cd app && cargo build

test:
	cd app && cargo test


