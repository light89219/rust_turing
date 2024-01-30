#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
