#!/bin/bash

# Upload Release Binaries to CCO Releases API
#
# Usage:
#   ./upload-release.sh <version> <binaries_directory>
#   ./upload-release.sh 2025.11.1 /path/to/artifacts
#
# Environment Variables:
#   CCO_API_URL: Base URL of CCO Releases API (default: https://cco-api.visiquate.com)
#   UPLOAD_API_KEY: API key for authentication
#   ACCESS_TOKEN: Authentik bearer token (for API authentication)

set -euo pipefail

# Configuration
VERSION="${1:?Error: Version not provided. Usage: $0 <version> <binaries_directory>}"
BINARIES_DIR="${2:?Error: Binaries directory not provided. Usage: $0 <version> <binaries_directory>}"
CCO_API_URL="${CCO_API_URL:-https://cco-api.visiquate.com}"
UPLOAD_API_KEY="${UPLOAD_API_KEY:?Error: UPLOAD_API_KEY environment variable not set}"
ACCESS_TOKEN="${ACCESS_TOKEN:?Error: ACCESS_TOKEN environment variable not set}"

# Validate inputs
if [ ! -d "$BINARIES_DIR" ]; then
    echo "Error: Binaries directory not found: $BINARIES_DIR"
    exit 1
fi

echo "Uploading CCO Release v${VERSION}"
echo "Source directory: ${BINARIES_DIR}"
echo "API URL: ${CCO_API_URL}"
echo ""

# Find all release binaries
BINARIES=$(find "$BINARIES_DIR" -maxdepth 1 -type f -name "cco-v${VERSION}-*" | sort)

if [ -z "$BINARIES" ]; then
    echo "Error: No binaries found matching pattern: cco-v${VERSION}-*"
    echo "Expected files in: ${BINARIES_DIR}"
    exit 1
fi

echo "Found binaries:"
echo "$BINARIES" | sed 's/^/  /'
echo ""

# Function to upload a file
upload_file() {
    local file="$1"
    local filename=$(basename "$file")

    echo "Uploading: ${filename}"

    # Upload file with API key authentication
    curl -X POST \
        -H "Authorization: Bearer ${ACCESS_TOKEN}" \
        -H "X-Upload-Key: ${UPLOAD_API_KEY}" \
        -F "file=@${file}" \
        -F "version=${VERSION}" \
        "${CCO_API_URL}/upload" \
        -w "\n"

    if [ $? -eq 0 ]; then
        echo "  ✓ Uploaded successfully"
    else
        echo "  ✗ Upload failed"
        return 1
    fi
}

# Upload all binaries
FAILED=0
for binary in $BINARIES; do
    if ! upload_file "$binary"; then
        FAILED=$((FAILED + 1))
    fi
done

echo ""
if [ $FAILED -eq 0 ]; then
    echo "✓ All files uploaded successfully"
    echo ""
    echo "Verify upload:"
    echo "  curl -H \"Authorization: Bearer \$ACCESS_TOKEN\" \\"
    echo "    ${CCO_API_URL}/releases/latest"
else
    echo "✗ ${FAILED} file(s) failed to upload"
    exit 1
fi
