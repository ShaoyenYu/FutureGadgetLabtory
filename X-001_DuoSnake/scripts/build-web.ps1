# PowerShell script to build and serve DuoSnake for Web (WASM)
param (
    [switch]$Serve,
    [switch]$Release,
    [int]$Port = 8080
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

Write-Host "=== DuoSnake Web (WASM) Build Helper ===" -ForegroundColor Cyan

# 1. Check if wasm32 target is installed
$installedTargets = rustup target list --installed
if ($installedTargets -notcontains "wasm32-unknown-unknown") {
    Write-Host "Installing wasm32-unknown-unknown target..." -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
}

# 2. Check if trunk is installed
$hasTrunk = Get-Command trunk -ErrorAction SilentlyContinue
if (-not $hasTrunk) {
    Write-Host "Trunk not found in PATH, checking ~/.cargo/bin..." -ForegroundColor Yellow
    $cargoBin = "$env:USERPROFILE\.cargo\bin"
    if (Test-Path "$cargoBin\trunk.exe") {
        $env:Path += ";$cargoBin"
    } else {
        Write-Host "Installing Trunk..." -ForegroundColor Yellow
        cargo install trunk --locked
    }
}

if ($Serve) {
    Write-Host "Starting Trunk development server on http://localhost:$Port ..." -ForegroundColor Green
    if ($Release) {
        trunk serve --release --port $Port
    } else {
        trunk serve --port $Port
    }
} else {
    Write-Host "Building WASM bundle with Trunk..." -ForegroundColor Green
    if ($Release) {
        trunk build --release
    } else {
        trunk build
    }
    Write-Host "Build completed! Output is in the 'dist' directory." -ForegroundColor Green
}
