param (
    [Parameter(Mandatory=$true, HelpMessage="The new version to apply (e.g. 1.26.1203.01)")]
    [string]$NewVersion
)

$ErrorActionPreference = "Stop"

Write-Host "Bumping version to $NewVersion across the project..." -ForegroundColor Cyan

# 1. Update src/lib.rs
$lib_path = "src/lib.rs"
$lib_content = Get-Content $lib_path -Raw
$lib_content = $lib_content -replace 'pub const VERSION: &str = ".*";', "pub const VERSION: &str = `"$NewVersion`";"
Set-Content -Path $lib_path -Value $lib_content -NoNewline
Write-Host " -> Updated $lib_path" -ForegroundColor Green

# 2. Update installer.iss
$iss_path = "installer.iss"
$iss_content = Get-Content $iss_path -Raw
$iss_content = $iss_content -replace '(?m)^AppVersion=.*$', "AppVersion=$NewVersion"
Set-Content -Path $iss_path -Value $iss_content -NoNewline
Write-Host " -> Updated $iss_path" -ForegroundColor Green

# 3. Update payload.json
$payload_path = "payload.json"
$payload_content = Get-Content $payload_path -Raw
$payload_content = $payload_content -replace '"app_version": ".*"', "`"app_version`": `"$NewVersion`""
Set-Content -Path $payload_path -Value $payload_content -NoNewline
Write-Host " -> Updated $payload_path" -ForegroundColor Green

Write-Host "`n✅ Version successfully updated to $NewVersion everywhere!" -ForegroundColor Green
Write-Host "Note: Do not forget to add a new entry for $NewVersion in 'src/ui/panels/release.rs' if you want it to appear in the changelog tab." -ForegroundColor Yellow
