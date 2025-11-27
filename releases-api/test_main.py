"""
Unit tests for FastAPI Releases API

Tests cover:
- Health check endpoint
- Latest release retrieval
- Specific version retrieval
- Download URL generation
- Error handling
- R2 connectivity verification
"""

import json
from unittest.mock import MagicMock, patch

import pytest
from fastapi.testclient import TestClient

from main import app


@pytest.fixture
def client():
    """FastAPI test client"""
    return TestClient(app)


class TestHealthCheck:
    """Health check endpoint tests"""

    def test_health_check_success(self, client):
        """Test successful health check with R2 connection"""
        with patch("main.r2_client.head_bucket") as mock_head:
            response = client.get("/health")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "healthy"
            assert data["service"] == "cco-releases-api"
            assert data["r2_connected"] is True
            mock_head.assert_called_once()

    def test_health_check_r2_failure(self, client):
        """Test health check with R2 connection failure"""
        with patch("main.r2_client.head_bucket") as mock_head:
            mock_head.side_effect = Exception("R2 connection failed")
            response = client.get("/health")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "degraded"
            assert data["r2_connected"] is False


class TestLatestRelease:
    """Latest release endpoint tests"""

    def test_get_latest_stable(self, client):
        """Test retrieving latest stable release"""
        mock_response = {
            "version": "2025.11.24",
            "channel": "stable",
            "released_at": "2025-11-24T10:30:00Z",
            "platforms": {
                "darwin-arm64": "abc123",
                "linux-x86_64": "def456",
            },
        }

        with patch("main.r2_client.get_object") as mock_get:
            mock_get.return_value = {
                "Body": MagicMock(read=MagicMock(return_value=json.dumps(mock_response).encode()))
            }
            response = client.get("/releases/latest?channel=stable")
            assert response.status_code == 200
            data = response.json()
            assert data["version"] == "2025.11.24"
            assert data["channel"] == "stable"
            mock_get.assert_called_with(
                Bucket="cco-releases-private", Key="metadata/latest-stable.json"
            )

    def test_get_latest_beta(self, client):
        """Test retrieving latest beta release"""
        mock_response = {
            "version": "2025.11.25",
            "channel": "beta",
            "released_at": "2025-11-25T10:30:00Z",
            "platforms": {"darwin-arm64": "xyz789"},
        }

        with patch("main.r2_client.get_object") as mock_get:
            mock_get.return_value = {
                "Body": MagicMock(read=MagicMock(return_value=json.dumps(mock_response).encode()))
            }
            response = client.get("/releases/latest?channel=beta")
            assert response.status_code == 200
            data = response.json()
            assert data["channel"] == "beta"

    def test_get_latest_invalid_channel(self, client):
        """Test invalid channel parameter"""
        response = client.get("/releases/latest?channel=invalid")
        assert response.status_code == 400
        assert "Invalid channel" in response.json()["detail"]

    def test_get_latest_not_found(self, client):
        """Test when latest release metadata not found"""
        with patch("main.r2_client.get_object") as mock_get:
            mock_get.side_effect = Exception("NoSuchKey")
            response = client.get("/releases/latest")
            assert response.status_code == 404

    def test_get_latest_r2_error(self, client):
        """Test R2 connection error"""
        with patch("main.r2_client.get_object") as mock_get:
            mock_get.side_effect = Exception("R2 connection error")
            response = client.get("/releases/latest")
            assert response.status_code == 500


class TestSpecificRelease:
    """Specific version release endpoint tests"""

    def test_get_release_by_version(self, client):
        """Test retrieving specific release version"""
        mock_response = {
            "version": "2025.11.24",
            "channel": "stable",
            "released_at": "2025-11-24T10:30:00Z",
            "platforms": {"darwin-arm64": "abc123"},
        }

        with patch("main.r2_client.get_object") as mock_get:
            mock_get.return_value = {
                "Body": MagicMock(read=MagicMock(return_value=json.dumps(mock_response).encode()))
            }
            response = client.get("/releases/2025.11.24")
            assert response.status_code == 200
            data = response.json()
            assert data["version"] == "2025.11.24"
            mock_get.assert_called_with(
                Bucket="cco-releases-private", Key="metadata/2025.11.24.json"
            )

    def test_get_release_not_found(self, client):
        """Test when specific release not found"""
        with patch("main.r2_client.get_object") as mock_get:
            mock_get.side_effect = Exception("NoSuchKey")
            response = client.get("/releases/9999.99.99")
            assert response.status_code == 404
            assert "not found" in response.json()["detail"]


class TestDownloadUrl:
    """Download URL generation tests"""

    @pytest.mark.parametrize(
        "platform,ext",
        [
            ("darwin-arm64", "tar.gz"),
            ("darwin-x86_64", "tar.gz"),
            ("linux-x86_64", "tar.gz"),
            ("linux-aarch64", "tar.gz"),
            ("windows-x86_64", "zip"),
        ],
    )
    def test_generate_presigned_url(self, client, platform, ext):
        """Test presigned URL generation for all platforms"""
        presigned_url = f"https://account.r2.cloudflarestorage.com/releases/2025.11.24/cco-v2025.11.24-{platform}.{ext}?token=abc123"

        with patch("main.r2_client.generate_presigned_url") as mock_gen:
            mock_gen.return_value = presigned_url
            response = client.get(f"/download/2025.11.24/{platform}")
            assert response.status_code == 200
            data = response.json()
            assert data["platform"] == platform
            assert data["version"] == "2025.11.24"
            assert data["download_url"] == presigned_url
            assert data["expires_in_seconds"] == 900
            mock_gen.assert_called_once()

    def test_invalid_platform(self, client):
        """Test invalid platform parameter"""
        response = client.get("/download/2025.11.24/invalid-platform")
        assert response.status_code == 400
        assert "Invalid platform" in response.json()["detail"]

    def test_presigned_url_expiry(self, client):
        """Test that presigned URL expires in 15 minutes (900 seconds)"""
        with patch("main.r2_client.generate_presigned_url") as mock_gen:
            mock_gen.return_value = "https://r2.example.com/file"
            response = client.get("/download/2025.11.24/darwin-arm64")
            data = response.json()
            assert data["expires_in_seconds"] == 900
            # Verify generate_presigned_url called with ExpiresIn=900
            call_kwargs = mock_gen.call_args[1]
            assert call_kwargs["ExpiresIn"] == 900

    def test_presigned_url_r2_error(self, client):
        """Test error handling during URL generation"""
        with patch("main.r2_client.generate_presigned_url") as mock_gen:
            mock_gen.side_effect = Exception("R2 error")
            response = client.get("/download/2025.11.24/darwin-arm64")
            assert response.status_code == 500
            assert "download URL" in response.json()["detail"]


class TestRootEndpoint:
    """Root endpoint tests"""

    def test_root_returns_404(self, client):
        """Test that root path returns 404"""
        response = client.get("/")
        assert response.status_code == 404
        assert "Not found" in response.json()["detail"]


class TestPlatformValidation:
    """Platform validation tests"""

    def test_valid_platforms(self, client):
        """Test that all valid platforms are accepted"""
        valid_platforms = [
            "darwin-arm64",
            "darwin-x86_64",
            "linux-x86_64",
            "linux-aarch64",
            "windows-x86_64",
        ]

        with patch("main.r2_client.generate_presigned_url") as mock_gen:
            mock_gen.return_value = "https://r2.example.com/file"
            for platform in valid_platforms:
                response = client.get(f"/download/2025.11.24/{platform}")
                assert response.status_code == 200, f"Platform {platform} should be valid"

    def test_invalid_platforms(self, client):
        """Test that invalid platforms are rejected"""
        invalid_platforms = ["macos-arm64", "mac-x64", "ubuntu-x64", "windows", "arm64"]

        for platform in invalid_platforms:
            response = client.get(f"/download/2025.11.24/{platform}")
            assert response.status_code == 400, f"Platform {platform} should be invalid"


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
