# Multi-stage build for FastAPI Releases API
FROM python:3.11-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements and install dependencies
COPY requirements.txt .
RUN pip install --user --no-cache-dir -r requirements.txt

# Final stage
FROM python:3.11-slim

WORKDIR /app

# Create non-root user
RUN groupadd -r releases-api && useradd -r -g releases-api releases-api

# Create releases directory with proper permissions
RUN mkdir -p /data/releases/metadata && \
    chown -R releases-api:releases-api /data/releases && \
    chmod -R 755 /data/releases

# Copy Python packages from builder
COPY --from=builder /root/.local /home/releases-api/.local
ENV PATH=/home/releases-api/.local/bin:$PATH

# Copy application code
COPY main.py .

# Set ownership
RUN chown -R releases-api:releases-api /app

# Switch to non-root user
USER releases-api

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Expose port
EXPOSE 8000

# Build arguments for metadata
ARG GIT_COMMIT=unknown
ARG BUILD_DATE=unknown
ARG VERSION=unknown

# Set environment variables
ENV GIT_COMMIT=${GIT_COMMIT}
ENV BUILD_DATE=${BUILD_DATE}
ENV VERSION=${VERSION}
ENV PYTHONUNBUFFERED=1
ENV ENVIRONMENT=production

# Start application
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
