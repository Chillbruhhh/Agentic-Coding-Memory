#!/usr/bin/env pwsh

# Test script for AMP codebase parsing functionality
# Tests parsing Python and TypeScript files and generating file logs

$ErrorActionPreference = "Stop"

Write-Host "🔍 Testing AMP Codebase Parser..." -ForegroundColor Cyan

# Check if server is running
$serverUrl = "http://localhost:8105"
try {
    $health = Invoke-RestMethod -Uri "$serverUrl/health" -Method GET
    Write-Host "✅ Server is running: $($health.service) v$($health.version)" -ForegroundColor Green
} catch {
    Write-Host "Server is not running. Please start the AMP server first." -ForegroundColor Red
    Write-Host "Run: cd amp/server; cargo run" -ForegroundColor Yellow
    exit 1
}

# Create test files
Write-Host "`n📁 Creating test files..." -ForegroundColor Yellow

$testDir = "test-codebase"
if (Test-Path $testDir) {
    Remove-Item -Recurse -Force $testDir
}
New-Item -ItemType Directory -Path $testDir | Out-Null

# Create Python test file
$pythonCode = @"
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
"@

$pythonCode | Out-File -FilePath "$testDir/user_manager.py" -Encoding UTF8

# Create TypeScript test file
$typescriptCode = @"
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
"@

$typescriptCode | Out-File -FilePath "$testDir/user_service.ts" -Encoding UTF8

Write-Host "✅ Created test files:" -ForegroundColor Green
Write-Host "  - $testDir/user_manager.py" -ForegroundColor Gray
Write-Host "  - $testDir/user_service.ts" -ForegroundColor Gray

# Test 1: Parse entire codebase
Write-Host "`n🔍 Test 1: Parse entire codebase..." -ForegroundColor Yellow

$parseCodebaseRequest = @{
    root_path = (Resolve-Path $testDir).Path
    project_id = "test-project"
    tenant_id = "test-tenant"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$serverUrl/v1/codebase/parse" -Method POST -Body $parseCodebaseRequest -ContentType "application/json"
    
    Write-Host "✅ Codebase parsing successful!" -ForegroundColor Green
    Write-Host "  Files parsed: $($response.files_parsed)" -ForegroundColor Gray
    Write-Host "Languages detected:" -ForegroundColor Gray
    
    foreach ($filePath in $response.file_logs.PSObject.Properties.Name) {
        $fileLog = $response.file_logs.$filePath
        $symbolCount = $fileLog.symbols.Count
        Write-Host "    - $($fileLog.path): $($fileLog.language) ($symbolCount symbols)" -ForegroundColor Gray
    }
} catch {
    Write-Host "Codebase parsing failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "Response: $($_.Exception.Response | ConvertTo-Json -Depth 3)" -ForegroundColor Red
}

# Test 2: Parse single Python file
Write-Host "`n🐍 Test 2: Parse Python file..." -ForegroundColor Yellow

$parseFileRequest = @{
    file_path = (Resolve-Path "$testDir/user_manager.py").Path
    language = "python"
    project_id = "test-project"
    tenant_id = "test-tenant"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$serverUrl/v1/codebase/parse-file" -Method POST -Body $parseFileRequest -ContentType "application/json"
    
    Write-Host "✅ Python file parsing successful!" -ForegroundColor Green
    Write-Host "  Symbols found: $($response.file_log.symbols.Count)" -ForegroundColor Gray
    
    foreach ($symbol in $response.file_log.symbols) {
        Write-Host "    - $($symbol.symbol_type): $($symbol.name) (lines $($symbol.start_line + 1)-$($symbol.end_line + 1))" -ForegroundColor Gray
    }
    
    Write-Host "`n📝 Generated Markdown:" -ForegroundColor Cyan
    Write-Host $response.markdown -ForegroundColor Gray
} catch {
    Write-Host "Python file parsing failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 3: Parse single TypeScript file
Write-Host "`n📘 Test 3: Parse TypeScript file..." -ForegroundColor Yellow

$parseFileRequest = @{
    file_path = (Resolve-Path "$testDir/user_service.ts").Path
    language = "typescript"
    project_id = "test-project"
    tenant_id = "test-tenant"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$serverUrl/v1/codebase/parse-file" -Method POST -Body $parseFileRequest -ContentType "application/json"
    
    Write-Host "✅ TypeScript file parsing successful!" -ForegroundColor Green
    Write-Host "  Symbols found: $($response.file_log.symbols.Count)" -ForegroundColor Gray
    
    foreach ($symbol in $response.file_log.symbols) {
        Write-Host "    - $($symbol.symbol_type): $($symbol.name) (lines $($symbol.start_line + 1)-$($symbol.end_line + 1))" -ForegroundColor Gray
    }
    
    Write-Host "`n📝 Generated Markdown:" -ForegroundColor Cyan
    Write-Host $response.markdown -ForegroundColor Gray
} catch {
    Write-Host "TypeScript file parsing failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: Update file log with changes
Write-Host "`n📝 Test 4: Update file log with changes..." -ForegroundColor Yellow

$updateRequest = @{
    file_path = (Resolve-Path "$testDir/user_manager.py").Path
    change_description = "Added error handling and logging"
    changeset_id = "cs_001"
    run_id = "run_001"
    decision_id = "dec_001"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "$serverUrl/v1/codebase/update-file-log" -Method POST -Body $updateRequest -ContentType "application/json"
    
    Write-Host "✅ File log update successful!" -ForegroundColor Green
    Write-Host "  Recent changes: $($response.file_log.recent_changes.Count)" -ForegroundColor Gray
    
    foreach ($change in $response.file_log.recent_changes) {
        Write-Host "    - $change" -ForegroundColor Gray
    }
    
    Write-Host "`n📝 Updated Markdown:" -ForegroundColor Cyan
    Write-Host $response.markdown -ForegroundColor Gray
} catch {
    Write-Host "File log update failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Get file log by path
Write-Host "`n📋 Test 5: Get file log by path..." -ForegroundColor Yellow

$encodedPath = [System.Web.HttpUtility]::UrlEncode((Resolve-Path "$testDir/user_manager.py").Path)

try {
    $response = Invoke-RestMethod -Uri "$serverUrl/v1/codebase/file-logs/$encodedPath" -Method GET
    
    Write-Host "✅ File log retrieval successful!" -ForegroundColor Green
    Write-Host "  File: $($response.file_log.path)" -ForegroundColor Gray
    Write-Host "  Language: $($response.file_log.language)" -ForegroundColor Gray
    Write-Host "  Hash: $($response.file_log.content_hash.Substring(0, 8))..." -ForegroundColor Gray
    Write-Host "  Symbols: $($response.file_log.symbols.Count)" -ForegroundColor Gray
} catch {
    Write-Host "File log retrieval failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Cleanup
Write-Host "`n🧹 Cleaning up test files..." -ForegroundColor Yellow
Remove-Item -Recurse -Force $testDir

Write-Host "`n🎉 Codebase parser testing complete!" -ForegroundColor Green
Write-Host "The AMP codebase parser can now:" -ForegroundColor Cyan
Write-Host "  Parse Python and TypeScript files" -ForegroundColor Gray
Write-Host "  Extract symbols (functions, classes, interfaces, etc.)" -ForegroundColor Gray
Write-Host "  Extract dependencies (imports/exports)" -ForegroundColor Gray
Write-Host "  Generate structured file logs in Markdown format" -ForegroundColor Gray
Write-Host "  Track file changes and link to decisions/changesets" -ForegroundColor Gray
Write-Host "  Compute content hashes for change detection" -ForegroundColor Gray
