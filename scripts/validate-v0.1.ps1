param(
  [switch]$FullBuild
)

$ErrorActionPreference = "Stop"

$scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Split-Path -Parent $scriptRoot

Push-Location $repoRoot
try {
  Write-Host "== cargo fmt voxverse"
  cargo fmt --manifest-path src-tauri\Cargo.toml --check
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }

  Write-Host "== cargo clippy"
  cargo clippy --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }

  Write-Host "== cargo test"
  cargo test --manifest-path src-tauri\Cargo.toml
  if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
  }

  $lv4Args = @()
  if ($FullBuild) {
    $lv4Args += "-FullBuild"
  }

  & powershell -NoProfile -ExecutionPolicy Bypass -File scripts\validate-lv4.ps1 @lv4Args
  exit $LASTEXITCODE
} finally {
  Pop-Location
}
