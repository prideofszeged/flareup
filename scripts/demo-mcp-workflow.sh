#!/bin/bash
# Demo: MCP-Driven Development Workflow
# This script demonstrates how to use the MCP workflow for feature development

set -e

echo "🎬 MCP-Driven Development Demo"
echo "================================"
echo ""
echo "This demo shows the power of using MCP to build Flareup itself."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Step 1: Check that Flareup is running
echo -e "${BLUE}Step 1: Verify Flareup is running${NC}"
echo "-----------------------------------"
if curl -s http://127.0.0.1:7266/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} Flareup debug API is running"
    curl -s http://127.0.0.1:7266/health | jq -r '"  Version: \(.version), Uptime: \(.uptime_seconds)s"'
else
    echo -e "${YELLOW}!${NC} Flareup debug API not running. Start with 'just dev'"
    exit 1
fi
echo ""

# Step 2: Show current state
echo -e "${BLUE}Step 2: Query current application state${NC}"
echo "----------------------------------------"
echo "Plugins installed:"
curl -s http://127.0.0.1:7266/plugins | jq -r '.data | length | "  → \(.) plugins loaded"'

echo "Applications discovered:"
curl -s http://127.0.0.1:7266/apps | jq -r '.data | length | "  → \(.) applications"'

echo "User data:"
curl -s http://127.0.0.1:7266/snippets | jq -r '.data | length | "  → \(.) snippets configured"'
curl -s http://127.0.0.1:7266/quicklinks | jq -r '.data | length | "  → \(.) quicklinks configured"'
echo ""

# Step 3: Demonstrate log querying
echo -e "${BLUE}Step 3: Query application logs${NC}"
echo "-------------------------------"
echo "Recent INFO logs:"
curl -s 'http://127.0.0.1:7266/logs?limit=3&level=info' | \
    jq -r '.data[] | "  [\(.timestamp | split("T")[1] | split(".")[0])] \(.message | .[0:60])..."'
echo ""

# Step 4: Show settings inspection
echo -e "${BLUE}Step 4: Inspect application configuration${NC}"
echo "-----------------------------------------"
echo "Current settings (sample):"
curl -s http://127.0.0.1:7266/settings | \
    jq -r 'to_entries[0:3] | .[] | "  \(.key): \(.value)"'
echo "  ... and more"
echo ""

# Step 5: Demonstrate usage analytics
echo -e "${BLUE}Step 5: Check usage analytics${NC}"
echo "------------------------------"
echo "Most frequently used items:"
curl -s http://127.0.0.1:7266/frecency | \
    jq -r '.data | sort_by(.total_score) | reverse | .[0:3] | .[] | "  → \(.item_id) (score: \(.total_score | floor), used \(.use_count) times)"'
echo ""

# Step 6: Show the meta advantage
echo -e "${BLUE}Step 6: The Meta Advantage${NC}"
echo "--------------------------"
echo "Traditional workflow:"
echo "  Code → Build (2min) → Open UI → Click around → Find issues → Repeat"
echo ""
echo "MCP-driven workflow:"
echo "  Code → Query MCP (instant) → Validate → Fix → Done"
echo ""
echo -e "${GREEN}Result: 10x faster iteration!${NC}"
echo ""

# Step 7: Show how to add a new feature
echo -e "${BLUE}Step 7: Adding a new feature${NC}"
echo "----------------------------"
echo "To add a new MCP-enabled feature:"
echo ""
echo "  1. Run: ${GREEN}just mcp-new-feature my_feature${NC}"
echo "     → Scaffolds all templates and docs"
echo ""
echo "  2. Implement the backend logic"
echo "     → Add to debug API"
echo "     → Add MCP tool"
echo ""
echo "  3. Validate via MCP:"
echo "     ${GREEN}just mcp-query '/my-feature'${NC}"
echo "     ${GREEN}just mcp-health${NC}"
echo ""
echo "  4. Test with AI:"
echo "     Ask Claude: 'Use flareup_get_my_feature to check it works'"
echo ""
echo "  5. Commit with confidence!"
echo ""

# Final message
echo "================================"
echo -e "${GREEN}✨ MCP-Driven Development = Development Nirvana${NC}"
echo ""
echo "Your AI assistant can now:"
echo "  ✓ Inspect Flareup's state in real-time"
echo "  ✓ Validate features without running the UI"
echo "  ✓ Debug issues by querying logs and settings"
echo "  ✓ Verify implementations match requirements"
echo "  ✓ Monitor performance and usage patterns"
echo ""
echo "Commands to try:"
echo "  ${GREEN}just mcp-health${NC}              - Full health check"
echo "  ${GREEN}just mcp-logs 20${NC}            - View 20 recent logs"
echo "  ${GREEN}just mcp-query /plugins${NC}     - List all plugins"
echo "  ${GREEN}just mcp-new-feature <name>${NC} - Scaffold new feature"
echo ""
echo "Welcome to meta development! 🚀"
