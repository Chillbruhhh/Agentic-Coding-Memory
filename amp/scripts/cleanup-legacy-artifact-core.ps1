# Remove the legacy global `artifact_core` hub and the edges that hang off it.
# Before find_or_create_artifact_core was scoped per-project, every artifact
# without a file link attached to a single global hub via `defined_in`. That
# hub is what visually bridges every project in the "All projects" view of
# the knowledge graph.
#
# This deletes:
#   - all `defined_in` edges where the target is an artifact_core node WITHOUT a project_id
#   - the legacy artifact_core node(s) themselves (those without a project_id)
#
# Per-project artifact_core nodes (created by the new code) are NOT touched.
#
# Usage:
#   .\cleanup-legacy-artifact-core.ps1                # dry run
#   .\cleanup-legacy-artifact-core.ps1 -Confirm       # actually deletes

param(
    [switch]$Confirm,
    [string]$BaseUrl = "http://localhost:8105"
)

Write-Host "=== Legacy Artifact Core Cleanup ===" -ForegroundColor Cyan
Write-Host ("Server: {0}" -f $BaseUrl) -ForegroundColor Gray
Write-Host ("Mode:   {0}" -f $(if ($Confirm) { "DELETE" } else { "DRY RUN" })) -ForegroundColor Gray
Write-Host ""

function Normalize-Id([string]$id) {
    if ([string]::IsNullOrEmpty($id)) { return "" }
    return ($id -replace "^objects:", "") -replace "[^0-9a-fA-F-]", ""
}

# Bulk fetch all objects.
Write-Host "Fetching objects..." -ForegroundColor Yellow
$queryBody = @{ limit = 100000 } | ConvertTo-Json
try {
    $queryResp = Invoke-RestMethod -Uri "$BaseUrl/v1/query" -Method Post -Body $queryBody -ContentType "application/json"
} catch {
    Write-Host ("Failed: {0}" -f $_.Exception.Message) -ForegroundColor Red
    exit 1
}
$objects = @()
if ($queryResp.results) { $objects = $queryResp.results | ForEach-Object { if ($_.object) { $_.object } else { $_ } } }
elseif ($queryResp -is [array]) { $objects = $queryResp }

# Find legacy artifact_core nodes (no project_id).
$legacyCores = $objects | Where-Object { $_.type -eq 'artifact_core' -and [string]::IsNullOrEmpty($_.project_id) }
Write-Host ("  legacy artifact_core nodes: {0}" -f $legacyCores.Count) -ForegroundColor Gray
if ($legacyCores.Count -eq 0) {
    Write-Host "Nothing to clean up." -ForegroundColor Green
    exit 0
}
$legacyCoreIds = @{}
foreach ($c in $legacyCores) {
    $key = Normalize-Id $c.id
    if ($key) { $legacyCoreIds[$key] = $c }
}

# Find edges to those legacy cores.
Write-Host "Fetching relationships..." -ForegroundColor Yellow
try {
    $edges = Invoke-RestMethod -Uri "$BaseUrl/v1/relationships" -Method Get
} catch {
    Write-Host ("Failed: {0}" -f $_.Exception.Message) -ForegroundColor Red
    exit 1
}

$edgesToDelete = @()
foreach ($edge in $edges) {
    $inKey  = Normalize-Id $edge.in
    $outKey = Normalize-Id $edge.out
    if ($legacyCoreIds.ContainsKey($inKey) -or $legacyCoreIds.ContainsKey($outKey)) {
        $edgesToDelete += [PSCustomObject]@{ id = $edge.id; type = $edge.type; inId = $edge.in; outId = $edge.out }
    }
}

Write-Host ""
Write-Host ("Edges hanging off legacy hub(s): {0}" -f $edgesToDelete.Count) -ForegroundColor Yellow
if ($edgesToDelete.Count -gt 0) {
    $byType = $edgesToDelete | Group-Object -Property type | Sort-Object Count -Descending
    foreach ($g in $byType) {
        Write-Host ("  {0,-15} {1}" -f $g.Name, $g.Count) -ForegroundColor Gray
    }
}

if (-not $Confirm) {
    Write-Host ""
    Write-Host "Dry run only. Re-run with -Confirm to delete." -ForegroundColor Yellow
    exit 0
}

# Delete edges first, then the core node(s).
Write-Host ""
Write-Host "Deleting edges..." -ForegroundColor Red
$edgeDel = 0
$edgeFail = 0
foreach ($e in $edgesToDelete) {
    try {
        Invoke-RestMethod -Uri "$BaseUrl/v1/relationships/$($e.type)/$($e.id)" -Method Delete | Out-Null
        $edgeDel++
    } catch {
        $edgeFail++
        Write-Host ("  failed edge {0}/{1}: {2}" -f $e.type, $e.id, $_.Exception.Message) -ForegroundColor Red
    }
}
Write-Host ("  edges deleted: {0}, failed: {1}" -f $edgeDel, $edgeFail) -ForegroundColor Green

Write-Host ""
Write-Host "Deleting legacy artifact_core nodes..." -ForegroundColor Red
$nodeDel = 0
$nodeFail = 0
foreach ($c in $legacyCores) {
    $key = Normalize-Id $c.id
    if (-not $key) { continue }
    try {
        Invoke-RestMethod -Uri "$BaseUrl/v1/objects/$key" -Method Delete | Out-Null
        $nodeDel++
    } catch {
        $nodeFail++
        Write-Host ("  failed node {0}: {1}" -f $key, $_.Exception.Message) -ForegroundColor Red
    }
}
Write-Host ("  nodes deleted: {0}, failed: {1}" -f $nodeDel, $nodeFail) -ForegroundColor Green
