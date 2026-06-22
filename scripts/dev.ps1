$env:Path = "C:\Program Files\nodejs;C:\Users\mghar\.cargo\bin;" + $env:Path
Set-Location (Split-Path $PSScriptRoot -Parent)
& npm run tauri dev
