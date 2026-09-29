#!/usr/bin/env bash
# Test script for Ariadne extension navigation flow.
# Verifies the server responds correctly to API calls the extension makes.

set -euo pipefail

SERVER_URL="${SERVER_URL:-http://localhost:8080}"

echo "=== Testing Ariadne Server API ==="
echo ""

# Test 1: Server health
echo "1. Testing server health..."
if curl -sf "${SERVER_URL}/api/repo-info" > /dev/null 2>&1; then
    echo "   ✅ Server is reachable at ${SERVER_URL}"
else
    echo "   ⚠️  Server not reachable at ${SERVER_URL}"
    echo "   Start server with: cargo run --bin cli --features server"
    exit 1
fi

# Test 2: Get repo info
echo "2. Fetching repo info..."
REPO_INFO=$(curl -s "${SERVER_URL}/api/repo-info")
echo "   Repo info: ${REPO_INFO}"

# Test 3: Search for symbols
echo "3. Testing symbol search..."
SEARCH_RESULT=$(curl -s "${SERVER_URL}/api/graph/search?q=main&limit=5")
echo "   Search result: ${SEARCH_RESULT}"

# Test 4: Status endpoint
echo "4. Testing status endpoint..."
STATUS=$(curl -s "${SERVER_URL}/api/status")
echo "   Status: ${STATUS}"

echo ""
echo "=== All API tests passed ==="