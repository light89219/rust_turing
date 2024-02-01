# Use when `cargo` fails with "failed to create directory ...\target" (Access is denied, Windows).
# Puts build output under %LOCALAPPDATA%\Rust\cargo-target-day3 instead of .\target
$dir = Join-Path $env:LOCALAPPDATA "Rust\cargo-target-day3"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$env:CARGO_TARGET_DIR = $dir
cargo @args
exit $LASTEXITCODE
