#!/usr/bin/env pwsh
# Verifies REST endpoints:
# - POST/GET /v1/cache/block/read
# - POST/GET /v1/cache/block/list
# - POST /v1/focus

$ErrorActionPreference = "Stop"
$SERVER_URL = "http://localhost:8105"

Write-Host "=== Testing REST Cache Read/List + Focus ===" -ForegroundColor Cyan

function PostJson($url, $obj) {
  $body = $obj | ConvertTo-Json -Depth 20
  return Invoke-RestMethod -Uri $url -Method Post -Body $body -ContentType "application/json"
}

Write-Host "`n[1] Writing cache item (should create or append to open block)..." -ForegroundColor Yellow
$writeResp = PostJson "$SERVER_URL/v1/cache/block/write" @{
  scope_id = "project:test"
  kind = "fact"
  content = "test"
}
Write-Host "write ok. block_id=$($writeResp.block_id)" -ForegroundColor Gray

Write-Host "`n[2] POST /v1/cache/block/read (list_all=true)..." -ForegroundColor Yellow
$readPost = PostJson "$SERVER_URL/v1/cache/block/read" @{
  scope_id = "project:test"
  list_all = $true
}
Write-Host "read(post) ok. matches=$($readPost.matches.Count)" -ForegroundColor Gray

Write-Host "`n[3] GET /v1/cache/block/read?scope_id=...&list_all=true..." -ForegroundColor Yellow
$readGet = Invoke-RestMethod -Uri "$SERVER_URL/v1/cache/block/read?scope_id=project%3Atest&list_all=true" -Method Get
Write-Host "read(get) ok. matches=$($readGet.matches.Count)" -ForegroundColor Gray

Write-Host "`n[4] GET /v1/cache/block/list?scope_id=...&limit=5..." -ForegroundColor Yellow
$listGet = Invoke-RestMethod -Uri "$SERVER_URL/v1/cache/block/list?scope_id=project%3Atest&limit=5" -Method Get
Write-Host "list(get) ok. matches=$($listGet.matches.Count)" -ForegroundColor Gray

Write-Host "`n[5] Compacting block..." -ForegroundColor Yellow
PostJson "$SERVER_URL/v1/cache/block/compact" @{ scope_id = "project:test" } | Out-Null
Write-Host "compact ok." -ForegroundColor Gray

Write-Host "`n[6] Creating run + testing /v1/focus set/get/complete/end..." -ForegroundColor Yellow
$run = PostJson "$SERVER_URL/v1/objects" @{
  type = "run"
  agent_name = "rest-test"
  goal = "REST focus route test"
  repo_id = "test-repo"
  tenant_id = "test-tenant"
  project_id = "test-project"
}
$runId = $run.id
Write-Host "run created. id=$runId" -ForegroundColor Gray

PostJson "$SERVER_URL/v1/focus" @{
  action = "set"
  run_id = $runId
  title = "Test focus"
  plan = @("step 1", "step 2")
  project_id = "test-project"
} | Out-Null
Write-Host "focus set ok." -ForegroundColor Gray

$focusGet = PostJson "$SERVER_URL/v1/focus" @{ action = "get"; run_id = $runId }
Write-Host "focus get ok. focus.title=$($focusGet.focus.title)" -ForegroundColor Gray

PostJson "$SERVER_URL/v1/focus" @{
  action = "complete"
  run_id = $runId
  summary = "done"
  files_changed = @("amp/server/src/main.rs")
} | Out-Null
Write-Host "focus complete ok." -ForegroundColor Gray

PostJson "$SERVER_URL/v1/focus" @{ action = "end"; run_id = $runId } | Out-Null
Write-Host "focus end ok." -ForegroundColor Gray

Write-Host "`n=== Done ===" -ForegroundColor Cyan

