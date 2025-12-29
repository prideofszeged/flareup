#!/bin/bash
# Test script for Flareup MCP Server
# This script tests all MCP server endpoints through the underlying HTTP debug API

echo "🔍 Flareup MCP Server Health Check"
echo "===================================="
echo ""

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BASE_URL="http://127.0.0.1:7266"

# Function to test an endpoint
test_endpoint() {
    local name=$1
    local endpoint=$2
    local description=$3
    
    echo -n "Testing $name... "
    response=$(curl -s -w "\n%{http_code}" "$BASE_URL$endpoint")
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "200" ]; then
        echo -e "${GREEN}✓${NC} $description"
        # Show a preview of the data
        if command -v jq &> /dev/null; then
            echo "$body" | jq -r 'if .success then "  → Success: \(.data | if type == "array" then "[\(length) items]" elif type == "object" then (keys | "{\(length) keys}") else . end)" else "  → Error: \(.error)" end' 2>/dev/null || echo "  → Response received"
        fi
    else
        echo -e "${RED}✗${NC} Failed (HTTP $http_code)"
    fi
    echo ""
}

# Test all endpoints
echo "1. Core Health"
echo "-------------"
test_endpoint "Health Check" "/health" "Server status and version"

echo "2. Application Data"
echo "------------------"
test_endpoint "Apps List" "/apps" "Installed applications"
test_endpoint "Plugins List" "/plugins" "Discovered plugins/extensions"

echo "3. User Data"
echo "-----------"
test_endpoint "Snippets" "/snippets" "Text expansion snippets"
test_endpoint "Quicklinks" "/quicklinks" "URL/file shortcuts"
test_endpoint "Aliases" "/aliases" "Command aliases"
test_endpoint "Frecency Data" "/frecency" "Usage frequency tracking"

echo "4. Configuration"
echo "---------------"
test_endpoint "Settings" "/settings" "Application settings"
test_endpoint "AI Settings" "/ai/settings" "AI configuration"

echo "5. Logging System"
echo "----------------"
test_endpoint "Log Config" "/logs/config" "Current log configuration"
test_endpoint "Recent Logs" "/logs?limit=5" "Last 5 log entries"

echo ""
echo "===================================="
echo "MCP Server Health Check Complete!"
echo ""

# Test MCP server can start
echo "6. MCP Server Process"
echo "--------------------"
echo -n "Testing MCP server startup... "
cd "$(dirname "$0")"
timeout 2 pnpm --dir packages/mcp-server dev < /dev/null > /tmp/mcp-test.log 2>&1
if grep -q "Flareup MCP server started" /tmp/mcp-test.log 2>/dev/null; then
    echo -e "${GREEN}✓${NC} Server starts successfully"
    echo "  → Log: $(grep "Flareup MCP server started" /tmp/mcp-test.log)"
else
    echo -e "${YELLOW}⚠${NC}  Could not verify server startup"
    echo "  → Check /tmp/mcp-test.log for details"
fi
echo ""

# Summary
echo "Summary:"
echo "--------"
echo "• Debug API running on: $BASE_URL"
echo "• MCP Server config: .mcp.json"
echo "• Server package: packages/mcp-server/"
echo ""
echo "To use the MCP server with Claude Desktop, add this to your config:"
echo ""
echo '  "flareup": {'
echo '    "command": "npx",'
echo '    "args": ["tsx", "packages/mcp-server/src/index.ts"],'
echo "    \"cwd\": \"$(pwd)\""
echo '  }'
