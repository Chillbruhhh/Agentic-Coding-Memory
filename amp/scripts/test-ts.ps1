#!/usr/bin/env pwsh

# Test TypeScript parsing

Write-Host "Testing TypeScript parsing..." -ForegroundColor Cyan

# Create TypeScript test file
$testDir = "test-ts"
if (Test-Path $testDir) {
    Remove-Item -Recurse -Force $testDir
}
New-Item -ItemType Directory -Path $testDir | Out-Null

$tsCode = @"
import { Component } from 'react';

interface User {
    name: string;
    age: number;
}

class UserService {
    private users: User[] = [];
    
    addUser(user: User): void {
        this.users.push(user);
    }
    
    getUsers(): User[] {
        return this.users;
    }
}

export function createUser(name: string, age: number): User {
    return { name, age };
}

export { UserService };
"@

$tsCode | Out-File -FilePath "$testDir/user.ts" -Encoding UTF8

Write-Host "Created user.ts" -ForegroundColor Green

# Test parsing
$parseRequest = @{
    file_path = (Resolve-Path "$testDir/user.ts").Path
    language = "typescript"
} | ConvertTo-Json

try {
    $response = Invoke-RestMethod -Uri "http://localhost:8105/v1/codebase/parse-file" -Method POST -Body $parseRequest -ContentType "application/json"
    
    Write-Host "SUCCESS: TypeScript file parsed!" -ForegroundColor Green
    Write-Host "Symbols found: $($response.file_log.symbols.Count)" -ForegroundColor Gray
    
    foreach ($symbol in $response.file_log.symbols) {
        Write-Host "  - $($symbol.symbol_type): $($symbol.name)" -ForegroundColor Gray
    }
    
    Write-Host "`nGenerated Markdown:" -ForegroundColor Cyan
    Write-Host $response.markdown -ForegroundColor White
    
} catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
}

# Cleanup
Remove-Item -Recurse -Force $testDir
Write-Host "`nTypeScript test complete!" -ForegroundColor Green
