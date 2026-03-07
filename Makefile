# This is also a sort of documentation =)
# most commands around here are useful during development or to understand rust

run:
	bacon run

test:
	cargo test

# creates the database, altough sqlite pretty much does it automatically
.PHONY: database
database:
	rm ./database/database.db || true
	rm ./database/database.db-wal || true
	sqlx database create
	sqlx migrate run
