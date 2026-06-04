# Portable Windows 10/11 x64 archive: dist/release/pixelprobe-<version>-windows-x64-portable.zip
# Runs on Windows with Rust (MSVC). The folder it prepares is also the input of msi.ps1.
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')

$target = 'x86_64-pc-windows-msvc'
# The one place of the version: `version` in [workspace.package] of Cargo.toml.
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value

cargo build --release --locked -p pixelprobe --target $target
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$name = "pixelprobe-$version-windows-x64-portable"
$stage = "dist/stage/$name"
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Force $stage, 'dist/release' | Out-Null
Copy-Item "target/$target/release/pixelprobe.exe" "$stage/pixelprobe.exe"
Copy-Item LICENSE-MIT "$stage/LICENSE-MIT.txt"
Copy-Item LICENSE-APACHE "$stage/LICENSE-APACHE.txt"
Compress-Archive -Path $stage -DestinationPath "dist/release/$name.zip" -Force
Write-Host "done: dist/release/$name.zip"
