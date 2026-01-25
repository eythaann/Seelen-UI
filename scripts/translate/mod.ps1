# Script specifically for i18n folders
$rootPath = "./src/static"
$allPaths = @()

# Find all i18n folders recursively
$i18nFolders = Get-ChildItem -Path $rootPath -Directory -Recurse -Filter "i18n"

if ($i18nFolders.Count -eq 0) {
    Write-Host "No 'i18n' folders found" -ForegroundColor Red
    exit
}

# Process each i18n folder and collect file paths
$i18nFolders | ForEach-Object {
    $i18nFolder = $_
    $i18nPath = $i18nFolder.FullName

    # Get all files inside the i18n folder
    Get-ChildItem -Path $i18nPath -File | ForEach-Object {
        $allPaths += $_.FullName
    }
}

# Display summary
Write-Host "`n📁 i18n folders found: $($i18nFolders.Count)" -ForegroundColor Green
Write-Host "📄 Total files in i18n: $($allPaths.Count)" -ForegroundColor Cyan

# Process each file with the translation tool
Write-Host "`n🚀 Starting translation process..." -ForegroundColor Yellow

$allPaths | ForEach-Object {
    $path = $_
    Write-Host ""
    Write-Host "🔧 Processing: $path" -ForegroundColor Magenta
    &".\target\debug\seelen-ui.exe" resource translate $path
}

Write-Host "`n✅ Translation process completed!" -ForegroundColor Green