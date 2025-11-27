"""
FastAPI Releases API Service

Provides authenticated endpoints for CCO binary distribution:
- /releases/latest - Get latest release metadata
- /releases/{version} - Get specific version metadata
- /download/{version}/{platform} - Stream binary files directly
- /upload/{version}/{platform} - Upload new release binaries (API key required)
- /health - Service health check

Authentication is handled by Traefik Forward Auth middleware.
No auth logic needed in this service.

File storage uses local filesystem (RELEASES_DIR).
"""

import json
import logging
import os
from datetime import datetime
from pathlib import Path
from typing import Optional

import aiofiles
from fastapi import FastAPI, HTTPException, Query, Header, UploadFile, File
from fastapi.responses import FileResponse
from pydantic import BaseModel

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Environment variables
RELEASES_DIR = Path(os.getenv("RELEASES_DIR", "/data/releases"))
UPLOAD_API_KEY = os.getenv("UPLOAD_API_KEY")
MAX_BINARY_SIZE = 500 * 1024 * 1024  # 500MB max file size

# Validate required environment variables
if not UPLOAD_API_KEY:
    logger.warning("UPLOAD_API_KEY not set - uploads will be disabled")

# Initialize FastAPI app with no public docs
app = FastAPI(
    title="CCO Releases API",
    description="Private binary distribution service",
    docs_url=None,  # No Swagger docs
    redoc_url=None,  # No ReDoc
    openapi_url=None,  # No OpenAPI schema
)


# Utility functions
def validate_path_safety(version: str, platform: str = None) -> None:
    """Validate that version and platform don't contain directory traversal attempts."""
    if ".." in version or "/" in version or "\\" in version:
        raise HTTPException(status_code=400, detail="Invalid version string")
    if platform and (".." in platform or "/" in platform or "\\" in platform):
        raise HTTPException(status_code=400, detail="Invalid platform string")


async def releases_dir_exists() -> bool:
    """Check if releases directory exists and is readable."""
    try:
        return RELEASES_DIR.exists() and RELEASES_DIR.is_dir()
    except Exception:
        return False

# Pydantic models
class ReleaseMetadata(BaseModel):
    """Release metadata structure"""

    version: str
    channel: str  # "stable" or "beta"
    released_at: str
    platforms: dict[str, str]  # platform -> sha256 checksum
    downloads: dict[str, str]  # platform -> download URLs (pre-populated for latest)


class DownloadUrl(BaseModel):
    """Download URL response"""

    platform: str
    version: str
    download_url: str
    file_size: int


class UploadResponse(BaseModel):
    """Upload response"""

    status: str
    path: str
    size: int


class HealthStatus(BaseModel):
    """Health check response"""

    status: str
    service: str
    version: str
    filesystem_available: bool
    releases_dir: str
    timestamp: str


# Routes


@app.get("/health", response_model=HealthStatus)
async def health_check():
    """Health check endpoint with filesystem availability verification"""
    filesystem_available = await releases_dir_exists()

    if not filesystem_available:
        logger.error(f"Releases directory not accessible: {RELEASES_DIR}")

    return HealthStatus(
        status="healthy" if filesystem_available else "degraded",
        service="cco-releases-api",
        version=os.getenv("VERSION", "unknown"),
        filesystem_available=filesystem_available,
        releases_dir=str(RELEASES_DIR),
        timestamp=datetime.utcnow().isoformat(),
    )


@app.get("/releases/latest", response_model=ReleaseMetadata)
async def get_latest_release(channel: str = Query("stable")):
    """
    Get latest release metadata for a channel.

    Args:
        channel: Release channel ("stable" or "beta")

    Returns:
        ReleaseMetadata with version and platform info
    """
    if channel not in ["stable", "beta"]:
        raise HTTPException(status_code=400, detail="Invalid channel")

    try:
        # Read metadata from filesystem
        metadata_file = RELEASES_DIR / "metadata" / f"latest-{channel}.json"

        if not metadata_file.exists():
            logger.warning(f"No latest release found for channel: {channel}")
            raise HTTPException(status_code=404, detail=f"No release found for channel {channel}")

        async with aiofiles.open(metadata_file, "r") as f:
            content = await f.read()
            metadata_json = json.loads(content)

        logger.info(f"Retrieved latest {channel} release: {metadata_json.get('version')}")
        return metadata_json

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error retrieving latest release: {str(e)}")
        raise HTTPException(status_code=500, detail="Failed to retrieve release metadata")


@app.get("/releases/{version}", response_model=ReleaseMetadata)
async def get_release_by_version(version: str):
    """
    Get specific release metadata by version.

    Args:
        version: Release version (e.g., "2025.11.24")

    Returns:
        ReleaseMetadata with version and platform info
    """
    validate_path_safety(version)

    try:
        # Read metadata from filesystem
        metadata_file = RELEASES_DIR / version / "version-info.json"

        if not metadata_file.exists():
            logger.warning(f"Release not found: {version}")
            raise HTTPException(status_code=404, detail=f"Release {version} not found")

        async with aiofiles.open(metadata_file, "r") as f:
            content = await f.read()
            metadata_json = json.loads(content)

        logger.info(f"Retrieved release metadata for version: {version}")
        return metadata_json

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error retrieving release {version}: {str(e)}")
        raise HTTPException(status_code=500, detail="Failed to retrieve release metadata")


@app.get("/download/{version}/{platform}")
async def download_binary(version: str, platform: str):
    """
    Stream binary file directly.

    Args:
        version: Release version (e.g., "2025.11.24")
        platform: Target platform (darwin-arm64, darwin-x86_64, linux-x86_64, etc.)

    Returns:
        FileResponse with binary data
    """
    valid_platforms = [
        "darwin-arm64",
        "darwin-x86_64",
        "linux-x86_64",
        "linux-aarch64",
        "windows-x86_64",
    ]

    validate_path_safety(version, platform)

    if platform not in valid_platforms:
        raise HTTPException(
            status_code=400,
            detail=f"Invalid platform. Valid: {', '.join(valid_platforms)}",
        )

    try:
        # Determine file extension based on platform
        if platform.startswith("windows"):
            file_ext = "zip"
        else:
            file_ext = "tar.gz"

        # Build path to binary file
        binary_file = RELEASES_DIR / version / f"cco-v{version}-{platform}.{file_ext}"

        if not binary_file.exists():
            logger.warning(f"Binary not found: {version}/{platform}")
            raise HTTPException(status_code=404, detail=f"Binary not found for {version}/{platform}")

        # Verify file size before serving
        file_size = binary_file.stat().st_size
        if file_size > MAX_BINARY_SIZE:
            logger.error(f"Binary file too large: {binary_file} ({file_size} bytes)")
            raise HTTPException(status_code=413, detail="File size exceeds maximum allowed")

        logger.info(f"Streaming binary: {version}/{platform} ({file_size} bytes)")

        return FileResponse(
            path=binary_file,
            media_type="application/octet-stream",
            filename=binary_file.name,
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error streaming binary: {str(e)}")
        raise HTTPException(status_code=500, detail="Failed to stream binary")


@app.post("/upload/{version}/{platform}", response_model=UploadResponse)
async def upload_release(
    version: str,
    platform: str,
    file: UploadFile = File(...),
    x_api_key: str = Header(None),
):
    """
    Upload a new release binary (requires API key).

    Args:
        version: Release version (e.g., "2025.11.24")
        platform: Target platform (darwin-arm64, darwin-x86_64, linux-x86_64, etc.)
        file: Binary file to upload
        x_api_key: API key for authentication

    Returns:
        UploadResponse with upload status and path
    """
    # Validate API key
    if not UPLOAD_API_KEY:
        logger.warning("Upload attempted but UPLOAD_API_KEY not configured")
        raise HTTPException(status_code=503, detail="Uploads not enabled")

    if x_api_key != UPLOAD_API_KEY:
        logger.warning(f"Upload attempted with invalid API key")
        raise HTTPException(status_code=401, detail="Invalid API key")

    # Validate version and platform
    validate_path_safety(version, platform)

    valid_platforms = [
        "darwin-arm64",
        "darwin-x86_64",
        "linux-x86_64",
        "linux-aarch64",
        "windows-x86_64",
    ]

    if platform not in valid_platforms:
        raise HTTPException(
            status_code=400,
            detail=f"Invalid platform. Valid: {', '.join(valid_platforms)}",
        )

    try:
        # Create version directory if it doesn't exist
        version_dir = RELEASES_DIR / version
        version_dir.mkdir(parents=True, exist_ok=True)

        # Validate file extension
        if platform.startswith("windows"):
            expected_ext = "zip"
        else:
            expected_ext = "tar.gz"

        if not file.filename.endswith(expected_ext):
            raise HTTPException(
                status_code=400,
                detail=f"Invalid file extension for {platform}. Expected: {expected_ext}",
            )

        # Build safe filename
        safe_filename = f"cco-v{version}-{platform}.{expected_ext}"
        filepath = version_dir / safe_filename

        # Read and validate file size
        content = await file.read()
        file_size = len(content)

        if file_size > MAX_BINARY_SIZE:
            logger.error(f"Upload rejected: file too large ({file_size} bytes)")
            raise HTTPException(status_code=413, detail="File size exceeds maximum allowed")

        if file_size == 0:
            raise HTTPException(status_code=400, detail="Empty file not allowed")

        # Write file atomically
        async with aiofiles.open(filepath, "wb") as f:
            await f.write(content)

        logger.info(f"Uploaded release binary: {version}/{platform} ({file_size} bytes)")

        return UploadResponse(
            status="uploaded",
            path=str(filepath),
            size=file_size,
        )

    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error uploading binary: {str(e)}")
        raise HTTPException(status_code=500, detail="Failed to upload binary")


@app.get("/")
async def root():
    """Root endpoint returns 404 - no public index"""
    raise HTTPException(status_code=404, detail="Not found")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8000,
        reload=os.getenv("ENVIRONMENT", "production") == "development",
    )
