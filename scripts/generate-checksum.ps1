$ErrorActionPreference = 'Stop'

$projectRoot = Split-Path -Parent $PSScriptRoot
$package = Get-Content -LiteralPath (Join-Path $projectRoot 'package.json') -Raw | ConvertFrom-Json
$tauriConfig = Get-Content -LiteralPath (Join-Path $projectRoot 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
$bundleDir = Join-Path $projectRoot 'src-tauri\target\release\bundle\nsis'
$pattern = "$($tauriConfig.productName)_$($package.version)_*-setup.exe"
$installers = @(Get-ChildItem -LiteralPath $bundleDir -Filter $pattern -File)

if ($installers.Count -ne 1) {
  throw "Expected exactly one NSIS installer matching '$pattern', found $($installers.Count)."
}

$installer = $installers[0]
$hash = (Get-FileHash -LiteralPath $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
$checksumPath = "$($installer.FullName).sha256"
$checksum = "$hash  $($installer.Name)"

[System.IO.File]::WriteAllText($checksumPath, "$checksum`n", [System.Text.Encoding]::ASCII)

$savedHash = ((Get-Content -LiteralPath $checksumPath -Raw).Trim() -split '\s+')[0]
if ($savedHash -ne $hash) {
  throw 'Checksum verification failed.'
}

Write-Host "Created: $checksumPath"
Write-Host "SHA-256: $hash"
