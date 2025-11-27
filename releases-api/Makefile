.PHONY: help build run dev test lint format clean setup

help:
	@echo "CCO Releases API - Development Commands"
	@echo ""
	@echo "Available targets:"
	@echo "  make setup        - Set up development environment"
	@echo "  make build        - Build Docker image"
	@echo "  make run          - Run production container"
	@echo "  make dev          - Run development with hot-reload"
	@echo "  make dev-minio    - Run with MinIO (local R2 simulation)"
	@echo "  make test         - Run tests"
	@echo "  make lint         - Run linter"
	@echo "  make format       - Format code"
	@echo "  make clean        - Clean up containers and cache"
	@echo "  make logs         - Show container logs"
	@echo "  make shell        - Open shell in container"
	@echo ""

setup:
	@echo "Setting up development environment..."
	cp .env.example .env
	python -m venv venv
	. venv/bin/activate && pip install -r requirements.txt
	@echo "Setup complete. Run: source venv/bin/activate"

build:
	@echo "Building Docker image..."
	docker build -t cco-releases-api:latest \
		--build-arg GIT_COMMIT=$$(git rev-parse --short HEAD) \
		--build-arg BUILD_DATE=$$(date -u +'%Y-%m-%dT%H:%M:%SZ') \
		--build-arg VERSION=$$(cat .env 2>/dev/null | grep VERSION | cut -d'=' -f2) \
		.
	@echo "Built successfully: cco-releases-api:latest"

run:
	@echo "Running production container..."
	docker compose up -d releases-api
	@sleep 2
	@echo "Service running at http://localhost:8000/health"

dev:
	@echo "Running development with hot-reload..."
	docker compose up releases-api

dev-minio:
	@echo "Running with MinIO (local R2 simulation)..."
	docker compose --profile dev up

test:
	@echo "Running tests..."
	docker compose exec releases-api pytest tests/ -v --cov

lint:
	@echo "Running linter..."
	docker compose exec releases-api pylint main.py

format:
	@echo "Formatting code..."
	docker compose exec releases-api black main.py
	docker compose exec releases-api isort main.py

clean:
	@echo "Cleaning up..."
	docker compose down -v
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	rm -rf .pytest_cache .coverage htmlcov
	@echo "Cleanup complete"

logs:
	@echo "Showing container logs..."
	docker compose logs -f releases-api

shell:
	@echo "Opening shell in container..."
	docker compose exec releases-api /bin/bash
