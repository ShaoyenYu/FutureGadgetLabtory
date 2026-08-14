# PowerShell script to build and run DuoSnake on Desktop (Windows)
param (
    [switch]$Release,
    [switch]$BuildOnly
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

Write-Host "=== DuoSnake Desktop (Native) Helper ===" -ForegroundColor Cyan

if ($BuildOnly) {
    if ($Release) {
        Write-Host "Building Desktop binary in release mode..." -ForegroundColor Green
        cargo build --release
    } else {
        Write-Host "Building Desktop binary in debug mode..." -ForegroundColor Green
        cargo build
    }
} else {
    if ($Release) {
        Write-Host "Running DuoSnake in release mode..." -ForegroundColor Green
        cargo run --release
    } else {
        Write-Host "Running DuoSnake in debug mode..." -ForegroundColor Green
        cargo run
    }
}
