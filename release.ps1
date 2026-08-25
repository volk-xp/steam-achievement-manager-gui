<#
.SYNOPSIS
    Builds, packages and publishes a GitHub release.

.DESCRIPTION
    Reads the version out of Cargo.toml, builds the release binary, zips it with
    the Steam DLL it needs, and publishes a GitHub release with that zip
    attached. Refuses to publish if anything is missing rather than shipping a
    broken download.

    Requires the GitHub CLI (gh) and that you have run `gh auth login` once.

.PARAMETER Dest
    Folder the built exe and DLL are copied to. Defaults to the one build.bat uses.

.PARAMETER Notes
    Release notes. A sensible default is used if you do not pass any.

.PARAMETER Draft
    Create the release as a draft so you can review it before it goes live.

.EXAMPLE
    .\release.ps1
    Builds and publishes a release tagged with the version in Cargo.toml.

.EXAMPLE
    .\release.ps1 -Draft -Notes "Fixes the sidebar count on first launch."
#>

[CmdletBinding()]
param(
    [string]$Dest = "C:\Users\MSI\Videos\Volk SAM",
    [string]$Notes,
    [switch]$Draft
)

$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot

# --- checks before we touch anything -----------------------------------------

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) {
    throw "The GitHub CLI is not installed, or this window was opened before you installed it. Run: winget install --id GitHub.cli -e   then open a new PowerShell window."
}

if (-not (Test-Path 'LICENSE')) {
    throw @"
There is no LICENSE file in this folder, so this release would be published
without the copyright notice it is required to carry.

This project is a fork. Download LICENSE from
https://github.com/mbwilding/steam-achievement-manager
put it in this folder, commit it, then run this again.
"@
}

$versionLine = Select-String -Path 'Cargo.toml' -Pattern '^version\s*=\s*"([^"]+)"' |
    Select-Object -First 1
if (-not $versionLine) {
    throw "Could not read the version from Cargo.toml."
}
$version = $versionLine.Matches[0].Groups[1].Value
$tag = "v$version"

# The tag will point at whatever you last pushed, not at your working folder, so
# uncommitted work would silently not be in the release.
$dirty = git status --porcelain 2>$null
if ($dirty) {
    Write-Warning "You have uncommitted changes. The release will be tagged against your last pushed commit, so those changes will not be in it."
}

$existing = gh release view $tag 2>$null
if ($LASTEXITCODE -eq 0) {
    throw "A release tagged $tag already exists. Bump the version in Cargo.toml, or delete that release with: gh release delete $tag"
}

# --- build -------------------------------------------------------------------

Write-Host "Building $version. The first build takes several minutes." -ForegroundColor Cyan
& "$PSScriptRoot\build.bat" $Dest
if ($LASTEXITCODE -ne 0) {
    throw "The build failed, so nothing was published. See BUILD.md."
}

$exe = Join-Path $Dest 'sam.exe'
$dll = Join-Path $Dest 'steam_api64.dll'
foreach ($file in @($exe, $dll)) {
    if (-not (Test-Path $file)) {
        throw "Expected $file after the build but it is not there. Nothing was published."
    }
}

# --- package -----------------------------------------------------------------

$zip = Join-Path $env:TEMP "sam-windows-x64-$version.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path $exe, $dll -DestinationPath $zip
Write-Host "Packaged $([math]::Round((Get-Item $zip).Length / 1MB, 1)) MB -> $zip" -ForegroundColor Cyan

# --- publish -----------------------------------------------------------------

if (-not $Notes) {
    $Notes = @"
Steam Achievement Manager $version, windowed build for Windows x64.

Download ``sam-windows-x64-$version.zip``, extract it, and run ``sam.exe``.
Keep ``steam_api64.dll`` in the same folder - the application will not start
without it.

Steam must be running and signed in.
"@
}

$ghArgs = @(
    'release', 'create', $tag, $zip,
    '--title', $version,
    '--notes', $Notes
)
if ($Draft) { $ghArgs += '--draft' }

& gh @ghArgs
if ($LASTEXITCODE -ne 0) {
    throw "gh could not create the release. The build and zip are fine - the zip is at $zip if you want to attach it by hand."
}

Write-Host ""
Write-Host "Published $tag." -ForegroundColor Green
gh release view $tag --web
