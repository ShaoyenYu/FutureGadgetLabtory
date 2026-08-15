# PowerShell script to build DuoSnake for Android
param (
    [switch]$Release,
    [switch]$Run
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

Write-Host "=== DuoSnake Android Build Helper ===" -ForegroundColor Cyan

# Configure standard Android SDK directory
$sdk = "$env:LOCALAPPDATA\Android\Sdk"
$env:ANDROID_HOME = $sdk
[Environment]::SetEnvironmentVariable("ANDROID_HOME", $sdk, "User")
[Environment]::SetEnvironmentVariable("ANDROID_HOME", $sdk, "Process")

# Set NDK (prefer latest NDK in ndk/ directory)
$ndkDir = Get-ChildItem "$sdk\ndk" -ErrorAction SilentlyContinue | Select-Object -Last 1
if ($ndkDir) {
    $env:NDK_HOME = $ndkDir.FullName
    $env:ANDROID_NDK_ROOT = $ndkDir.FullName
} elseif (Test-Path "$sdk\ndk-bundle") {
    $env:NDK_HOME = "$sdk\ndk-bundle"
    $env:ANDROID_NDK_ROOT = "$sdk\ndk-bundle"
}
[Environment]::SetEnvironmentVariable("NDK_HOME", $env:NDK_HOME, "User")
[Environment]::SetEnvironmentVariable("NDK_HOME", $env:NDK_HOME, "Process")
[Environment]::SetEnvironmentVariable("ANDROID_NDK_ROOT", $env:NDK_HOME, "User")
[Environment]::SetEnvironmentVariable("ANDROID_NDK_ROOT", $env:NDK_HOME, "Process")

# Configure Java Home
$jbr = "C:\Program Files\JetBrains\PyCharm 2025.3\jbr"
if (Test-Path $jbr) {
    $env:JAVA_HOME = $jbr
    $env:Path = "$jbr\bin;" + $env:Path
    [Environment]::SetEnvironmentVariable("JAVA_HOME", $jbr, "User")
}

# Ensure Cargo Bin is in Path
$cargoBin = "$env:USERPROFILE\.cargo\bin"
if ($env:Path -notlike "*$cargoBin*") {
    $env:Path = "$cargoBin;" + $env:Path
}

Write-Host "ANDROID_HOME:     $env:ANDROID_HOME" -ForegroundColor DarkGray
Write-Host "NDK_HOME:         $env:NDK_HOME" -ForegroundColor DarkGray
Write-Host "ANDROID_NDK_ROOT: $env:ANDROID_NDK_ROOT" -ForegroundColor DarkGray
Write-Host "JAVA_HOME:        $env:JAVA_HOME" -ForegroundColor DarkGray

if ($Run) {
    Write-Host "Running APK on connected Android device..." -ForegroundColor Green
    if ($Release) {
        cargo apk run --lib --release
    } else {
        cargo apk run --lib
    }
} else {
    Write-Host "Building Android APK..." -ForegroundColor Green
    if ($Release) {
        cargo apk build --lib --release
    } else {
        cargo apk build --lib
    }
}
