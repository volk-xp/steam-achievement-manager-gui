<#
.SYNOPSIS
    Builds, packages and publishes a GitHub release.

.DESCRIPTION
    Reads the version out of Cargo.toml, builds the release binary, zips it with
    the Steam DLL it needs, and publishes a GitHub release with that zip
    attached. Checks the things that can go wrong before starting the build, so
    a problem costs you seconds rather than ten minutes.

    Requires git and the GitHub CLI, and that you have run `gh auth login` once.

.PARAMETER Dest
    Folder the built exe and DLL are copied to. Defaults to the one build.bat uses.

.PARAMETER Notes
    Release notes. A sensible default is used if you do not pass any.

.PARAMETER Draft
    Create the release as a draft so you can review it before it goes live.

.EXAMPLE
    .\release.ps1

.EXAMPLE
    .\release.ps1 -Draft -Notes "Fixes the sidebar count on first launch."
#>

[CmdletBinding()]
param(
    [string]$Dest = "C:\Users\MSI\Videos\Volk SAM",
    [string]$Notes,
    [switch]$Draft
)

# Deliberately NOT 'Stop'. PowerShell 5.1 turns anything a native command writes
# to stderr into an error record, and cargo sends all its build progress there,
# git sends hints there, and `gh release view` sends "release not found" there --
# all of which are normal, expected output. Under 'Stop' the script dies on them.
# Exit codes are checked explicitly instead, which is what actually matters.
$ErrorActionPreference = 'Continue'

Set-Location $PSScriptRoot

function Fail([string]$Message) {
    Write-Host ''
    Write-Host $Message -ForegroundColor Red
    Write-Host ''
    exit 1
}

function Step([string]$Message) {
    Write-Host ''
    Write-Host $Message -ForegroundColor Cyan
}

# Runs a native command, swallowing every stream, and returns its exit code.
# Used for the "does this exist?" probes where failure is a normal answer.
function Probe([scriptblock]$Command) {
    & $Command *>$null
    return $LASTEXITCODE
}

# --- checks, cheapest and most likely first ----------------------------------

foreach ($tool in 'git', 'gh') {
    if (-not (Get-Command $tool -ErrorAction SilentlyContinue)) {
        Fail "$tool is not installed, or this window was opened before you installed it.`nRun:  winget install --id $(if ($tool -eq 'gh') { 'GitHub.cli' } else { 'Git.Git' }) -e`nThen open a new PowerShell window."
    }
}

if (-not (Test-Path 'Cargo.toml')) {
    Fail "No Cargo.toml here. Run this from the project folder."
}

if (-not (Test-Path 'LICENSE')) {
    Fail @"
There is no LICENSE file in this folder, so this release would go out without the
copyright notice it has to carry.

This project is a fork. Download LICENSE from
  https://github.com/mbwilding/steam-achievement-manager
put it in this folder, commit it, then run this again.
"@
}

$versionLine = Select-String -Path 'Cargo.toml' -Pattern '^version\s*=\s*"([^"]+)"' |
    Select-Object -First 1
if (-not $versionLine) {
    Fail "Could not find a version line in Cargo.toml."
}
$version = $versionLine.Matches[0].Groups[1].Value
$tag = "v$version"

# Confirm the repo exists on GitHub now, rather than after a ten-minute build.
if ((Probe { gh repo view --json name }) -ne 0) {
    Fail @"
This folder is not connected to a GitHub repository yet, or gh is not signed in.

  gh auth login
  gh repo create steam-achievement-manager-gui --private --source . --remote origin --push
"@
}

if ((Probe { gh release view $tag }) -eq 0) {
    Fail "A release tagged $tag already exists.`nBump the version in Cargo.toml, or remove it with:  gh release delete $tag --cleanup-tag"
}

# The tag points at your last pushed commit, not at your working folder.
$dirty = & git status --porcelain 2>$null
if ($dirty) {
    Write-Warning "You have uncommitted changes. They will NOT be in this release, because the tag is created from your last pushed commit."
}

# --- build -------------------------------------------------------------------

Step "Building $version. The first build takes several minutes."

& "$PSScriptRoot\build.bat" $Dest
if ($LASTEXITCODE -ne 0) {
    Fail "The build failed, so nothing was published. See BUILD.md for what each error means."
}

$exe = Join-Path $Dest 'sam.exe'
$dll = Join-Path $Dest 'steam_api64.dll'
foreach ($file in @($exe, $dll)) {
    if (-not (Test-Path -LiteralPath $file)) {
        Fail "Expected $file after the build, but it is not there. Nothing was published."
    }
}

# --- package -----------------------------------------------------------------

Step "Packaging"

$zip = Join-Path $env:TEMP "sam-windows-x64-$version.zip"
try {
    if (Test-Path -LiteralPath $zip) { Remove-Item -LiteralPath $zip -Force -ErrorAction Stop }
    Compress-Archive -LiteralPath $exe, $dll -DestinationPath $zip -ErrorAction Stop
} catch {
    Fail "Could not build the zip: $($_.Exception.Message)"
}

$size = [math]::Round((Get-Item -LiteralPath $zip).Length / 1MB, 1)
Write-Host "  $zip  ($size MB)"

# --- publish -----------------------------------------------------------------

if (-not $Notes) {
    # Single-quoted here-string: no backtick escapes, no accidental interpolation.
    $template = @'
Steam Achievement Manager {0}, windowed build for Windows x64.

Download the zip below and extract it, then run sam.exe. Keep steam_api64.dll in
the same folder - the application will not start without it.

Steam must be running and signed in.
'@
    $Notes = $template -f $version
}

Step "Publishing $tag"

$ghArgs = @('release', 'create', $tag, $zip, '--title', $version, '--notes', $Notes)
if ($Draft) { $ghArgs += '--draft' }

& gh @ghArgs
if ($LASTEXITCODE -ne 0) {
    Fail "gh could not create the release. The build and the zip are fine -- it is at`n  $zip`nif you want to attach it by hand."
}

Write-Host ''
Write-Host "Published $tag." -ForegroundColor Green
& gh release view $tag --web
