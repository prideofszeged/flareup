#!/bin/bash
# Quick script to scaffold a new MCP-enabled feature

set -e

if [ -z "$1" ]; then
    echo "Usage: ./scripts/new-mcp-feature.sh <feature_name>"
    echo "Example: ./scripts/new-mcp-feature.sh clipboard_history"
    exit 1
fi

FEATURE_NAME=$1
FEATURE_CAMEL=$(echo $FEATURE_NAME | sed -r 's/(^|_)([a-z])/\U\2/g')
FEATURE_SNAKE=$(echo $FEATURE_NAME | tr '[:upper:]' '[:lower:]' | tr '-' '_')

echo "🚀 Creating MCP-enabled feature: $FEATURE_NAME"
echo ""

# 1. Create feature planning doc
echo "📝 Step 1: Creating feature plan..."
cat > "docs/${FEATURE_SNAKE}_FEATURE_PLAN.md" <<EOF
# $FEATURE_CAMEL Feature Plan

## Overview
[Describe what this feature does]

## User Stories
- As a user, I want to...

## Technical Design

### Backend (Rust)
- [ ] Implement feature logic in \`src-tauri/src/${FEATURE_SNAKE}.rs\`
- [ ] Add debug API endpoint: \`/api/debug/${FEATURE_SNAKE//_/-}\`
- [ ] Add Tauri commands for frontend integration

### Frontend (Svelte)
- [ ] Create UI component
- [ ] Add state management
- [ ] Integrate with command palette

### MCP Integration
- [ ] Add debug endpoint to expose state
- [ ] Create MCP tool: \`flareup_get_${FEATURE_SNAKE}\`
- [ ] Add to test script
- [ ] Document MCP usage

## Debug API Endpoint

\`\`\`
GET /api/debug/${FEATURE_SNAKE//_/-}
\`\`\`

Response:
\`\`\`json
{
  "success": true,
  "data": {
    // Define your data structure
  }
}
\`\`\`

## MCP Tool

**Name**: \`flareup_get_${FEATURE_SNAKE}\`

**Description**: [Describe what AI assistants can do with this]

**Usage Example**:
\`\`\`
Use flareup_get_${FEATURE_SNAKE} to [do something]
\`\`\`

## Testing Plan
- [ ] Unit tests
- [ ] Integration tests via MCP
- [ ] Manual testing checklist

## Validation via MCP
\`\`\`bash
./test-mcp-server.sh
# Should show: ✓ ${FEATURE_CAMEL} endpoint working
\`\`\`
EOF

echo "  ✓ Created docs/${FEATURE_SNAKE}_FEATURE_PLAN.md"

# 2. Create MCP tool template
echo ""
echo "🔧 Step 2: Creating MCP tool code templates..."

cat > "/tmp/${FEATURE_SNAKE}_mcp_tool.ts" <<EOF
// Add to TOOLS array in packages/mcp-server/src/index.ts
{
    name: 'flareup_get_${FEATURE_SNAKE}',
    description: 'TODO: Describe what this tool does',
    inputSchema: {
        type: 'object',
        properties: {
            // Add parameters if needed
        },
        required: [],
    },
},

// Add to client.ts
async get${FEATURE_CAMEL}(): Promise<ApiResponse<${FEATURE_CAMEL}Data>> {
    return this.fetch<${FEATURE_CAMEL}Data>('/${FEATURE_SNAKE//_/-}');
}

// Add to executeTool() switch statement
case 'flareup_get_${FEATURE_SNAKE}': {
    const result = await flareupClient.get${FEATURE_CAMEL}();
    if (result.success && result.data) {
        return JSON.stringify(result.data, null, 2);
    }
    return \`Error getting ${FEATURE_SNAKE}: \${result.error}\`;
}
EOF

echo "  ✓ Created MCP tool templates in /tmp/${FEATURE_SNAKE}_mcp_tool.ts"

# 3. Create debug API endpoint template
cat > "/tmp/${FEATURE_SNAKE}_debug_endpoint.rs" <<EOF
// Add to src-tauri/src/debug_api.rs

#[derive(Debug, Serialize, Deserialize)]
pub struct ${FEATURE_CAMEL}Data {
    // TODO: Define your data structure
    pub example_field: String,
}

async fn debug_get_${FEATURE_SNAKE}(
    state: State<'_, AppState>,
) -> Result<impl Reply, Rejection> {
    // TODO: Implement data retrieval
    let data = ${FEATURE_CAMEL}Data {
        example_field: "Hello from ${FEATURE_NAME}".to_string(),
    };
    
    Ok(warp::reply::json(&ApiResponse::success(data)))
}

// Add to debug_router():
.or(warp::path!("${FEATURE_SNAKE//_/-}")
    .and(warp::get())
    .and(with_state(app_state.clone()))
    .and_then(debug_get_${FEATURE_SNAKE}))
EOF

echo "  ✓ Created debug API template in /tmp/${FEATURE_SNAKE}_debug_endpoint.rs"

# 4. Update test script
echo ""
echo "🧪 Step 3: Adding to test script..."

# Create backup
cp test-mcp-server.sh test-mcp-server.sh.bak

# Add test before the summary section
sed -i "/^echo \"Summary:\"/i\\
echo \"Testing ${FEATURE_CAMEL}\"\\
echo \"-------------\"\\
test_endpoint \"${FEATURE_CAMEL}\" \"/${FEATURE_SNAKE//_/-}\" \"${FEATURE_NAME} data\"\\
echo \"\"\\
" test-mcp-server.sh

echo "  ✓ Updated test-mcp-server.sh (backup saved as .bak)"

# 5. Create checklist
echo ""
echo "📋 Step 4: Creating implementation checklist..."
cat > "/tmp/${FEATURE_SNAKE}_checklist.md" <<EOF
# ${FEATURE_CAMEL} Implementation Checklist

## Backend
- [ ] Create \`src-tauri/src/${FEATURE_SNAKE}.rs\`
- [ ] Implement core feature logic
- [ ] Add debug endpoint (see /tmp/${FEATURE_SNAKE}_debug_endpoint.rs)
- [ ] Add Tauri commands for frontend
- [ ] Add structured logging
- [ ] Handle errors gracefully

## Frontend
- [ ] Create UI component
- [ ] Add state management
- [ ] Connect to Tauri commands
- [ ] Add to command palette (if applicable)

## MCP Integration
- [ ] Add MCP tool definition (see /tmp/${FEATURE_SNAKE}_mcp_tool.ts)
- [ ] Add client method
- [ ] Add tool handler
- [ ] Test endpoint works: \`curl http://127.0.0.1:7266/${FEATURE_SNAKE//_/-}\`

## Testing
- [ ] Run \`./test-mcp-server.sh\` - should pass
- [ ] Test with Claude via MCP
- [ ] Add unit tests
- [ ] Test error cases

## Documentation
- [ ] Complete feature plan in docs/${FEATURE_SNAKE}_FEATURE_PLAN.md
- [ ] Add MCP usage examples
- [ ] Update README if needed

## Validation
- [ ] Feature works via UI
- [ ] Debug endpoint returns correct data
- [ ] MCP tool accessible
- [ ] No errors in logs
- [ ] Test script passes
EOF

echo "  ✓ Created /tmp/${FEATURE_SNAKE}_checklist.md"

# Summary
echo ""
echo "✨ Feature scaffolding complete!"
echo ""
echo "Created files:"
echo "  📄 docs/${FEATURE_SNAKE}_FEATURE_PLAN.md"
echo "  📄 /tmp/${FEATURE_SNAKE}_mcp_tool.ts"
echo "  📄 /tmp/${FEATURE_SNAKE}_debug_endpoint.rs"
echo "  📄 /tmp/${FEATURE_SNAKE}_checklist.md"
echo ""
echo "Next steps:"
echo "  1. Review and complete docs/${FEATURE_SNAKE}_FEATURE_PLAN.md"
echo "  2. Implement backend using template in /tmp/${FEATURE_SNAKE}_debug_endpoint.rs"
echo "  3. Add MCP tools using template in /tmp/${FEATURE_SNAKE}_mcp_tool.ts"
echo "  4. Follow checklist in /tmp/${FEATURE_SNAKE}_checklist.md"
echo "  5. Run ./test-mcp-server.sh to validate"
echo ""
echo "Happy meta development! 🚀"
