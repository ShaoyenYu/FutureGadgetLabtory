# PowerShell script to build DuoSnake for Android
param (
    [switch]$Release,
    [switch]$Run
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

Write-Host "=== DuoSnake Android Build Helper ===" -ForegroundColor Cyan

# Check for cargo-apk
$hasCargoApk = Get-Command cargo-apk -ErrorAction SilentlyContinue
if (-not $hasCargoApk) {
    Write-Host "cargo-apk not found. Install with: cargo install cargo-apk" -ForegroundColor Yellow
}

if ($Run) {
    Write-Host "Running APK on connected Android device..." -ForegroundColor Green
    if ($Release) {
        cargo apk run --release
    } else {
        cargo apk run
    }
} else {
    Write-Host "Building Android APK..." -ForegroundColor Green
    if ($Release) {
        cargo apk build --release
    } else {
        cargo apk build
    }
}
