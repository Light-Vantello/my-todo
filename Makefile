build:
	docker-compose -f .\docker\docker-compose.yml build

db:
	docker-compose -f .\docker\docker-compose.yml up

dev:
	sqlx db create
	sqlx migrate run
	cargo watch -x run

test:
	cargo test