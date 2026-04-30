# Find and delete graph edges whose endpoints belong to different projects.
# These are leftovers from before find_file_node_id and find_or_create_artifact_core
# were scoped by project_id.
#
# Usage:
#   .\cleanup-cross-project-edges.ps1                 # dry run, prints summary
#   .\cleanup-cross-project-edges.ps1 -Confirm        # actually deletes
#   .\cleanup-cross-project-edges.ps1 -BaseUrl ...    # override server URL

param(
    [switch]$Confirm,
    [string]$BaseUrl = "http://localhost:8105"
)

Write-Host "=== AMP Cross-Project Edge Cleanup ===" -ForegroundColor Cyan
Write-Host ("Server: {0}" -f $BaseUrl) -ForegroundColor Gray
Write-Host ("Mode:   {0}" -f $(if ($Confirm) { "DELETE" } else { "DRY RUN" })) -ForegroundColor Gray
Write-Host ""

# Strip "objects:" prefix and SurrealDB's UUID brackets to get a bare UUID.
function Normalize-Id([string]$id) {
    if ([string]::IsNullOrEmpty($id)) { return "" }
    return ($id -replace "^objects:", "") -replace "[^0-9a-fA-F-]", ""
}

# 1. Bulk-fetch all objects (one call instead of thousands).
Write-Host "Fetching all objects..." -ForegroundColor Yellow
$queryBody = @{ limit = 100000 } | ConvertTo-Json
try {
    $queryResp = Invoke-RestMethod -Uri "$BaseUrl/v1/query" -Method Post -Body $queryBody -ContentType "application/json"
} catch {
    Write-Host ("Failed to fetch objects: {0}" -f $_.Exception.Message) -ForegroundColor Red
    exit 1
}
$objects = @()
if ($queryResp.results) {
    $objects = $queryResp.results | ForEach-Object { if ($_.object) { $_.object } else { $_ } }
} elseif ($queryResp -is [array]) {
    $objects = $queryResp
}
$byId = @{}
foreach ($obj in $objects) {
    if (-not $obj.id) { continue }
    $key = Normalize-Id $obj.id
    if ($key) { $byId[$key] = $obj }
}
Write-Host ("  indexed {0} objects" -f $byId.Count) -ForegroundColor Gray

# 2. Fetch every edge.
Write-Host "Fetching relationships..." -ForegroundColor Yellow
try {
    $edges = Invoke-RestMethod -Uri "$BaseUrl/v1/relationships" -Method Get
} catch {
    Write-Host ("Failed to fetch relationships: {0}" -f $_.Exception.Message) -ForegroundColor Red
    exit 1
}
Write-Host ("  total edges: {0}" -f $edges.Count) -ForegroundColor Gray

# 3. Find cross-project edges. Only flag when BOTH endpoints have non-empty
# project_ids AND those project_ids differ. Edges touching project-less nodes
# are left to a separate cleanup pass.
$crossProjectEdges = @()
foreach ($edge in $edges) {
    $inKey  = Normalize-Id $edge.in
    $outKey = Normalize-Id $edge.out
    $inObj  = if ($byId.ContainsKey($inKey))  { $byId[$inKey]  } else { $null }
    $outObj = if ($byId.ContainsKey($outKey)) { $byId[$outKey] } else { $null }
    if ($null -eq $inObj -or $null -eq $outObj) { continue }
    $inPid  = $inObj.project_id
    $outPid = $outObj.project_id
    if ([string]::IsNullOrEmpty($inPid) -or [string]::IsNullOrEmpty($outPid)) { continue }
    if ($inPid -ne $outPid) {
        $crossProjectEdges += [PSCustomObject]@{
            id      = $edge.id
            type    = $edge.type
            inId    = $edge.in
            outId   = $edge.out
            inProj  = $inPid
            outProj = $outPid
        }
    }
}

# 4. Summary.
Write-Host ""
Write-Host ("Found {0} cross-project edges" -f $crossProjectEdges.Count) -ForegroundColor Yellow
if ($crossProjectEdges.Count -eq 0) {
    Write-Host "Nothing to clean up." -ForegroundColor Green
    exit 0
}

$byType = $crossProjectEdges | Group-Object -Property type | Sort-Object Count -Descending
Write-Host ""
Write-Host "By relation type:" -ForegroundColor Cyan
foreach ($g in $byType) {
    Write-Host ("  {0,-15} {1}" -f $g.Name, $g.Count) -ForegroundColor Gray
}

Write-Host ""
Write-Host "Sample (first 5):" -ForegroundColor Cyan
$crossProjectEdges | Select-Object -First 5 | ForEach-Object {
    Write-Host ("  [{0}] {1}({2}) -> {3}({4})" -f $_.type, $_.inId, $_.inProj, $_.outId, $_.outProj) -ForegroundColor DarkGray
}

# 5. Delete if -Confirm.
if (-not $Confirm) {
    Write-Host ""
    Write-Host "Dry run only. Re-run with -Confirm to delete." -ForegroundColor Yellow
    exit 0
}

Write-Host ""
Write-Host "Deleting..." -ForegroundColor Red
$deleted = 0
$failed  = 0
foreach ($e in $crossProjectEdges) {
    try {
        Invoke-RestMethod -Uri "$BaseUrl/v1/relationships/$($e.type)/$($e.id)" -Method Delete | Out-Null
        $deleted++
    } catch {
        $failed++
        Write-Host ("  failed: {0}/{1} -- {2}" -f $e.type, $e.id, $_.Exception.Message) -ForegroundColor Red
    }
}
Write-Host ""
Write-Host ("Deleted: {0}" -f $deleted) -ForegroundColor Green
if ($failed -gt 0) {
    Write-Host ("Failed:  {0}" -f $failed) -ForegroundColor Red
}
