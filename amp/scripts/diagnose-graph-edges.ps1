# Read-only diagnostic: classify every edge by what's on each endpoint.
# Use this to figure out what's actually visually bridging projects in the graph.
#
# Buckets:
#   same-project     : both endpoints have the same non-empty project_id (clean)
#   cross-project    : both have non-empty project_ids that differ (bug — should be 0 after fix)
#   one-empty        : one endpoint has project_id, the other doesn't (suspect — usually a hub edge)
#   both-empty       : neither endpoint has project_id (legacy, mostly artifact_core wiring)
#   missing-endpoint : one or both endpoints couldn't be matched to a known object
#
# Usage: powershell -ExecutionPolicy Bypass -File scripts/diagnose-graph-edges.ps1

param(
    [string]$BaseUrl = "http://localhost:8105"
)

Write-Host "=== AMP Graph Edge Diagnostic ===" -ForegroundColor Cyan
Write-Host ("Server: {0}" -f $BaseUrl) -ForegroundColor Gray
Write-Host ""

# Strip "objects:" prefix and SurrealDB's UUID brackets to get a bare UUID.
function Normalize-Id([string]$id) {
    if ([string]::IsNullOrEmpty($id)) { return "" }
    return ($id -replace "^objects:", "") -replace "[^0-9a-fA-F-]", ""
}

# 1. Fetch ALL objects in one bulk query (much faster than 4000+ individual GETs).
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
    $objects = $queryResp.results | ForEach-Object {
        if ($_.object) { $_.object } else { $_ }
    }
} elseif ($queryResp -is [array]) {
    $objects = $queryResp
}
Write-Host ("  fetched {0} objects" -f $objects.Count) -ForegroundColor Gray

# Build id -> object map for O(1) lookup.
$byId = @{}
foreach ($obj in $objects) {
    if (-not $obj.id) { continue }
    $key = Normalize-Id $obj.id
    if ($key) { $byId[$key] = $obj }
}
Write-Host ("  indexed {0} ids" -f $byId.Count) -ForegroundColor Gray

# 2. Fetch every edge.
Write-Host ""
Write-Host "Fetching relationships..." -ForegroundColor Yellow
try {
    $edges = Invoke-RestMethod -Uri "$BaseUrl/v1/relationships" -Method Get
} catch {
    Write-Host ("Failed: {0}" -f $_.Exception.Message) -ForegroundColor Red
    exit 1
}
Write-Host ("  total edges: {0}" -f $edges.Count) -ForegroundColor Gray
Write-Host ""

# 3. Classify.
$buckets = @{
    'same-project'     = [System.Collections.ArrayList]@()
    'cross-project'    = [System.Collections.ArrayList]@()
    'one-empty'        = [System.Collections.ArrayList]@()
    'both-empty'       = [System.Collections.ArrayList]@()
    'missing-endpoint' = [System.Collections.ArrayList]@()
}

foreach ($edge in $edges) {
    $inKey  = Normalize-Id $edge.in
    $outKey = Normalize-Id $edge.out
    $inObj  = if ($byId.ContainsKey($inKey))  { $byId[$inKey]  } else { $null }
    $outObj = if ($byId.ContainsKey($outKey)) { $byId[$outKey] } else { $null }

    if ($null -eq $inObj -or $null -eq $outObj) {
        [void]$buckets['missing-endpoint'].Add([PSCustomObject]@{
            type = $edge.type; inId = $edge.in; outId = $edge.out
            inFound = ($null -ne $inObj); outFound = ($null -ne $outObj)
        })
        continue
    }

    $inPid  = $inObj.project_id
    $outPid = $outObj.project_id
    $inEmpty  = [string]::IsNullOrEmpty($inPid)
    $outEmpty = [string]::IsNullOrEmpty($outPid)

    $row = [PSCustomObject]@{
        type = $edge.type; inKind = $inObj.kind; outKind = $outObj.kind
        inName = $inObj.name; outName = $outObj.name
        inProj = $inPid; outProj = $outPid
    }

    if ($inEmpty -and $outEmpty) {
        [void]$buckets['both-empty'].Add($row)
    } elseif ($inEmpty -or $outEmpty) {
        [void]$buckets['one-empty'].Add($row)
    } elseif ($inPid -ne $outPid) {
        [void]$buckets['cross-project'].Add($row)
    } else {
        [void]$buckets['same-project'].Add($row)
    }
}

Write-Host "=== Bucket counts ===" -ForegroundColor Cyan
foreach ($name in 'same-project','cross-project','one-empty','both-empty','missing-endpoint') {
    $count = $buckets[$name].Count
    $color = switch ($name) {
        'same-project'     { 'Green' }
        'cross-project'    { 'Red' }
        'one-empty'        { 'Yellow' }
        'both-empty'       { 'Yellow' }
        'missing-endpoint' { 'Magenta' }
    }
    Write-Host ("  {0,-18} {1}" -f $name, $count) -ForegroundColor $color
}

# Drill into suspect buckets.
foreach ($bucketName in 'one-empty','both-empty','cross-project') {
    $list = $buckets[$bucketName]
    if ($list.Count -eq 0) { continue }
    Write-Host ""
    Write-Host ("=== {0}: by endpoint kind pair (top 10) ===" -f $bucketName) -ForegroundColor Cyan
    $list |
        Group-Object -Property { "{0} -[{1}]-> {2}" -f $_.inKind, $_.type, $_.outKind } |
        Sort-Object Count -Descending |
        Select-Object -First 10 |
        ForEach-Object {
            Write-Host ("  {0,5}  {1}" -f $_.Count, $_.Name) -ForegroundColor Gray
        }

    Write-Host ""
    Write-Host ("=== {0}: sample (first 5) ===" -f $bucketName) -ForegroundColor Cyan
    $list | Select-Object -First 5 | ForEach-Object {
        if ($bucketName -eq 'cross-project') {
            Write-Host ("  [{0}] {1}({2})/{3} -> {4}({5})/{6}" -f $_.type, $_.inName, $_.inKind, $_.inProj, $_.outName, $_.outKind, $_.outProj) -ForegroundColor DarkGray
        } else {
            Write-Host ("  [{0}] {1}({2})/{3} -> {4}({5})/{6}" -f $_.type, $_.inName, $_.inKind, $(if ($_.inProj) { $_.inProj } else { '<none>' }), $_.outName, $_.outKind, $(if ($_.outProj) { $_.outProj } else { '<none>' })) -ForegroundColor DarkGray
        }
    }
}

# Special: list endpoints involved in 'one-empty' edges that have no project_id —
# these are the actual hub nodes bridging projects in "All projects" view.
if ($buckets['one-empty'].Count -gt 0) {
    Write-Host ""
    Write-Host "=== Top 'orphan' nodes (no project_id) involved in one-empty edges ===" -ForegroundColor Cyan
    $orphanNodes = @{}
    foreach ($row in $buckets['one-empty']) {
        $orphan = if ([string]::IsNullOrEmpty($row.inProj))  { "{0} ({1})" -f $row.inName, $row.inKind }
                  else                                       { "{0} ({1})" -f $row.outName, $row.outKind }
        if (-not $orphanNodes.ContainsKey($orphan)) { $orphanNodes[$orphan] = 0 }
        $orphanNodes[$orphan]++
    }
    $orphanNodes.GetEnumerator() |
        Sort-Object -Property Value -Descending |
        Select-Object -First 10 |
        ForEach-Object {
            Write-Host ("  {0,5}  {1}" -f $_.Value, $_.Key) -ForegroundColor Gray
        }
}

Write-Host ""
Write-Host "Done." -ForegroundColor Green
