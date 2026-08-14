# Forwarding wrapper to scripts/build-web.ps1
param (
    [switch]$Serve,
    [switch]$Release,
    [int]$Port = 8080
)

& "$PSScriptRoot\scripts\build-web.ps1" -Serve:$Serve -Release:$Release -Port $Port
