# Test Artifacts

This directory contains sample artifacts for testing the AMP Artifacts UI feature. These JSON files demonstrate all four artifact types supported by AMP.

## Artifact Types

### 1. Decisions (decisions.json)
Architectural Decision Records (ADRs) documenting key technical choices.

**Fields:**
- `title`: Decision name
- `status`: accepted | proposed | deprecated
- `context`: Background and problem statement
- `decision`: The actual decision made
- `consequences`: Positive and negative implications
- `alternatives`: Other options considered
- `linked_files`: Files affected by this decision

**UI Display:**
- Icon: Lightbulb (amber)
- Shows status badge
- Displays all decision sections

### 2. File Logs (filelogs.json)
Documentation of file structure, symbols, and change history.

**Fields:**
- `title`: File description
- `file_path`: Path to the file
- `summary`: Overview of file purpose
- `symbols`: Key functions, classes, etc.
- `dependencies`: Imported modules
- `change_history`: Timeline of modifications

**UI Display:**
- Icon: Document (blue)
- Shows file path in monospace
- Lists symbols as badges
- Timeline of changes

### 3. Notes (notes.json)
General observations, TODOs, questions, and warnings.

**Fields:**
- `title`: Note title
- `category`: insight | todo | question | warning
- `content`: Markdown-formatted content
- `linked_objects`: Related artifact IDs

**UI Display:**
- Icon: Note (green)
- Shows category badge
- Renders markdown content
- Supports code blocks and lists

### 4. Change Sets (changesets.json)
Records of code modifications and their impact.

**Fields:**
- `title`: Change description
- `description`: Detailed explanation
- `diff_summary`: Git-style diff summary
- `files_changed`: List of modified files
- `linked_decisions`: Decisions that justify changes

**UI Display:**
- Icon: Git branch (purple)
- Shows diff summary in monospace
- Lists all changed files
- Links to related decisions

## Loading Test Artifacts

### Option 1: Via API (Recommended)

Use the AMP CLI or direct API calls to import artifacts:

```bash
# Using AMP CLI
cd test-repo/artifacts
amp artifact import decisions.json
amp artifact import notes.json
amp artifact import changesets.json
amp artifact import filelogs.json

# Or via curl
curl -X POST http://localhost:8105/v1/artifacts \
  -H "Content-Type: application/json" \
  -d @decisions.json
```

### Option 2: Via UI (Future)

The Artifacts UI will eventually support drag-and-drop import of JSON files.

### Option 3: Via Script

Create a PowerShell script to batch import:

```powershell
# scripts/import-test-artifacts.ps1
$artifacts = @(
    "test-repo/artifacts/decisions.json",
    "test-repo/artifacts/notes.json",
    "test-repo/artifacts/changesets.json",
    "test-repo/artifacts/filelogs.json"
)

foreach ($file in $artifacts) {
    Write-Host "Importing $file..."
    $content = Get-Content $file -Raw | ConvertFrom-Json
    
    foreach ($artifact in $content) {
        $json = $artifact | ConvertTo-Json -Depth 10
        Invoke-RestMethod -Uri "http://localhost:8105/v1/artifacts" `
            -Method Post `
            -ContentType "application/json" `
            -Body $json
    }
}

Write-Host "Import complete!"
```

## Expected Results

After importing all artifacts, the Artifacts UI should show:

- **ALL tab**: 28 total artifacts
- **Decisions**: 3 artifacts (amber icons)
- **File Logs**: 10 artifacts (blue icons)
- **Notes**: 5 artifacts (green icons)
- **Change Sets**: 5 artifacts (purple icons)

Each artifact should:
- Display in the left panel with correct icon and color
- Show memory layer badges (graph, vector, temporal)
- Open detailed view when clicked
- Render type-specific content correctly

## Testing Checklist

- [ ] All 28 artifacts import successfully
- [ ] Correct counts in each tab
- [ ] Icons and colors match artifact types
- [ ] Memory layer badges display correctly
- [ ] Detail panels show all fields
- [ ] Markdown rendering works in notes
- [ ] Code blocks display properly
- [ ] Timestamps format correctly
- [ ] Tags display as badges
- [ ] Linked files/decisions show relationships

## Troubleshooting

**Artifacts not showing in UI:**
- Check server logs for import errors
- Verify database connection
- Ensure artifacts have required fields (type, title)

**Memory layer badges missing:**
- Check if embedding service is enabled
- Verify vector layer is working
- Ensure graph relationships were created

**Detail panel empty:**
- Check artifact ID in browser console
- Verify API endpoint returns full artifact
- Check for JavaScript errors in console

## Related Files

- `amp/ui/src/components/Artifacts.tsx` - UI component
- `amp/server/src/handlers/artifacts.rs` - Backend handlers
- `amp/server/src/models/` - Artifact data models
- `test-repo/` - Sample code files referenced by artifacts
