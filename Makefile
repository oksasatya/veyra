.PHONY: dev db-up db-down test lint fmt check

dev:
	cd apps/backend && cargo run

db-up:
	docker compose -f docker-compose.dev.yml up -d

db-down:
	docker compose -f docker-compose.dev.yml down

test:
	cd apps/backend && cargo nextest run

lint:
	cd apps/backend && cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cd apps/backend && cargo fmt --check

check: fmt lint test
