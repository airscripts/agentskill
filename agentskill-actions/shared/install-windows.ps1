$ErrorActionPreference = 'Stop'

if ([string]::IsNullOrWhiteSpace($env:AGENTSKILL_VERSION)) {
  throw 'Agentskill version is required when source mode is disabled.'
}

$target = switch ("$env:RUNNER_ARCH") {
  'X64' { 'x86_64-pc-windows-msvc' }
  'ARM64' { 'aarch64-pc-windows-msvc' }
  default { throw "Unsupported Agentskill runner: $env:RUNNER_OS-$env:RUNNER_ARCH" }
}

$releaseVersion = $env:AGENTSKILL_VERSION -replace '-rc\..*$', ''
$package = "agentskill-$releaseVersion-$target"
$archivePath = Join-Path $env:RUNNER_TEMP "$package.zip"
$checksumsPath = Join-Path $env:RUNNER_TEMP 'agentskill/SHA256SUMS'
$installDir = Join-Path $env:RUNNER_TEMP 'agentskill/bin'
New-Item -ItemType Directory -Force -Path $installDir | Out-Null

Invoke-WebRequest -Uri "https://github.com/airscripts/agentskill/releases/download/$env:AGENTSKILL_VERSION/$package.zip" -OutFile $archivePath
Invoke-WebRequest -Uri "https://github.com/airscripts/agentskill/releases/download/$env:AGENTSKILL_VERSION/SHA256SUMS" -OutFile $checksumsPath

$expectedChecksum = Get-Content $checksumsPath |
  Where-Object { $_ -match "\s$([regex]::Escape($package))$" } |
  ForEach-Object { ($_ -split '\s+')[0] } |
  Select-Object -First 1
if ([string]::IsNullOrWhiteSpace($expectedChecksum)) { throw "Checksum is missing for $package" }

$actualChecksum = (Get-FileHash $archivePath -Algorithm SHA256).Hash
if ($actualChecksum.ToLowerInvariant() -ne $expectedChecksum.ToLowerInvariant()) {
  throw "Checksum verification failed for $package"
}

Expand-Archive -LiteralPath $archivePath -DestinationPath $env:RUNNER_TEMP -Force
Copy-Item (Join-Path $env:RUNNER_TEMP "$package/agentskill.exe") (Join-Path $installDir 'agentskill.exe')
Add-Content -Path $env:GITHUB_PATH -Value $installDir
