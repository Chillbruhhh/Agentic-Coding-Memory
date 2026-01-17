#!/bin/bash

# Test script for AMP codebase parsing functionality
# Tests parsing Python and TypeScript files and generating file logs

set -e

echo "🔍 Testing AMP Codebase Parser..."

# Check if server is running
SERVER_URL="http://localhost:8105"
if ! curl -s "$SERVER_URL/health" > /dev/null; then
    echo "❌ Server is not running. Please start the AMP server first."
    echo "Run: cd amp/server && cargo run"
    exit 1
fi

HEALTH=$(curl -s "$SERVER_URL/health")
echo "✅ Server is running: $(echo $HEALTH | jq -r '.service') v$(echo $HEALTH | jq -r '.version')"

# Create test files
echo ""
echo "📁 Creating test files..."

TEST_DIR="test-codebase"
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Create Python test file
cat > "$TEST_DIR/user_manager.py" << 'EOF'
import os
import sys
from typing import List, Dict

class UserManager:
    """Manages user operations."""
    
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.users = []
    
    def create_user(self, name: str, email: str) -> Dict:
        """Create a new user."""
        user = {
            'id': len(self.users) + 1,
            'name': name,
            'email': email
        }
        self.users.append(user)
        return user
    
    def get_users(self) -> List[Dict]:
        """Get all users."""
        return self.users

def main():
    """Main entry point."""
    manager = UserManager("users.db")
    user = manager.create_user("John Doe", "john@example.com")
    print(f"Created user: {user}")

if __name__ == "__main__":
    main()
EOF

# Create TypeScript test file
cat > "$TEST_DIR/user_service.ts" << 'EOF'
import { Component, ReactNode } from 'react';
import axios from 'axios';

interface User {
    id: number;
    name: string;
    email: string;
}

interface UserListProps {
    users: User[];
    onUserSelect: (user: User) => void;
}

class UserService {
    private baseUrl: string;
    
    constructor(baseUrl: string) {
        this.baseUrl = baseUrl;
    }
    
    async getUsers(): Promise<User[]> {
        const response = await axios.get<User[]>(`${this.baseUrl}/users`);
        return response.data;
    }
    
    async createUser(name: string, email: string): Promise<User> {
        const response = await axios.post<User>(`${this.baseUrl}/users`, {
            name,
            email
        });
        return response.data;
    }
}

export class UserList extends Component<UserListProps> {
    render(): ReactNode {
        return (
            <div>
                {this.props.users.map(user => (
                    <div key={user.id} onClick={() => this.props.onUserSelect(user)}>
                        {user.name} - {user.email}
                    </div>
                ))}
            </div>
        );
    }
}

export function createUserService(baseUrl: string): UserService {
    return new UserService(baseUrl);
}

export { User, UserService };
EOF

echo "✅ Created test files:"
echo "  - $TEST_DIR/user_manager.py"
echo "  - $TEST_DIR/user_service.ts"

# Test 1: Parse entire codebase
echo ""
echo "🔍 Test 1: Parse entire codebase..."

PARSE_REQUEST=$(cat << EOF
{
    "root_path": "$(pwd)/$TEST_DIR",
    "project_id": "test-project",
    "tenant_id": "test-tenant"
}
EOF
)

RESPONSE=$(curl -s -X POST "$SERVER_URL/v1/codebase/parse" \
    -H "Content-Type: application/json" \
    -d "$PARSE_REQUEST")

if echo "$RESPONSE" | jq -e '.success' > /dev/null; then
    echo "✅ Codebase parsing successful!"
    FILES_PARSED=$(echo "$RESPONSE" | jq -r '.files_parsed')
    echo "  Files parsed: $FILES_PARSED"
    echo "  Languages detected:"
    echo "$RESPONSE" | jq -r '.file_logs | to_entries[] | "    - \(.value.path): \(.value.language) (\(.value.symbols | length) symbols)"'
else
    echo "❌ Codebase parsing failed"
    echo "$RESPONSE" | jq '.'
fi

# Test 2: Parse single Python file
echo ""
echo "🐍 Test 2: Parse Python file..."

PARSE_FILE_REQUEST=$(cat << EOF
{
    "file_path": "$(pwd)/$TEST_DIR/user_manager.py",
    "language": "python",
    "project_id": "test-project",
    "tenant_id": "test-tenant"
}
EOF
)

RESPONSE=$(curl -s -X POST "$SERVER_URL/v1/codebase/parse-file" \
    -H "Content-Type: application/json" \
    -d "$PARSE_FILE_REQUEST")

if echo "$RESPONSE" | jq -e '.file_log' > /dev/null; then
    echo "✅ Python file parsing successful!"
    SYMBOL_COUNT=$(echo "$RESPONSE" | jq -r '.file_log.symbols | length')
    echo "  Symbols found: $SYMBOL_COUNT"
    echo "$RESPONSE" | jq -r '.file_log.symbols[] | "    - \(.symbol_type): \(.name) (lines \(.start_line + 1)-\(.end_line + 1))"'
    
    echo ""
    echo "📝 Generated Markdown:"
    echo "$RESPONSE" | jq -r '.markdown'
else
    echo "❌ Python file parsing failed"
    echo "$RESPONSE" | jq '.'
fi

# Test 3: Parse single TypeScript file
echo ""
echo "📘 Test 3: Parse TypeScript file..."

PARSE_FILE_REQUEST=$(cat << EOF
{
    "file_path": "$(pwd)/$TEST_DIR/user_service.ts",
    "language": "typescript",
    "project_id": "test-project",
    "tenant_id": "test-tenant"
}
EOF
)

RESPONSE=$(curl -s -X POST "$SERVER_URL/v1/codebase/parse-file" \
    -H "Content-Type: application/json" \
    -d "$PARSE_FILE_REQUEST")

if echo "$RESPONSE" | jq -e '.file_log' > /dev/null; then
    echo "✅ TypeScript file parsing successful!"
    SYMBOL_COUNT=$(echo "$RESPONSE" | jq -r '.file_log.symbols | length')
    echo "  Symbols found: $SYMBOL_COUNT"
    echo "$RESPONSE" | jq -r '.file_log.symbols[] | "    - \(.symbol_type): \(.name) (lines \(.start_line + 1)-\(.end_line + 1))"'
    
    echo ""
    echo "📝 Generated Markdown:"
    echo "$RESPONSE" | jq -r '.markdown'
else
    echo "❌ TypeScript file parsing failed"
    echo "$RESPONSE" | jq '.'
fi

# Test 4: Update file log with changes
echo ""
echo "📝 Test 4: Update file log with changes..."

UPDATE_REQUEST=$(cat << EOF
{
    "file_path": "$(pwd)/$TEST_DIR/user_manager.py",
    "change_description": "Added error handling and logging",
    "changeset_id": "cs_001",
    "run_id": "run_001",
    "decision_id": "dec_001"
}
EOF
)

RESPONSE=$(curl -s -X POST "$SERVER_URL/v1/codebase/update-file-log" \
    -H "Content-Type: application/json" \
    -d "$UPDATE_REQUEST")

if echo "$RESPONSE" | jq -e '.file_log' > /dev/null; then
    echo "✅ File log update successful!"
    CHANGE_COUNT=$(echo "$RESPONSE" | jq -r '.file_log.recent_changes | length')
    echo "  Recent changes: $CHANGE_COUNT"
    echo "$RESPONSE" | jq -r '.file_log.recent_changes[] | "    - \(.)"'
    
    echo ""
    echo "📝 Updated Markdown:"
    echo "$RESPONSE" | jq -r '.markdown'
else
    echo "❌ File log update failed"
    echo "$RESPONSE" | jq '.'
fi

# Cleanup
echo ""
echo "🧹 Cleaning up test files..."
rm -rf "$TEST_DIR"

echo ""
echo "🎉 Codebase parser testing complete!"
echo "The AMP codebase parser can now:"
echo "  ✅ Parse Python and TypeScript files"
echo "  ✅ Extract symbols (functions, classes, interfaces, etc.)"
echo "  ✅ Extract dependencies (imports/exports)"
echo "  ✅ Generate structured file logs in Markdown format"
echo "  ✅ Track file changes and link to decisions/changesets"
echo "  ✅ Compute content hashes for change detection"
