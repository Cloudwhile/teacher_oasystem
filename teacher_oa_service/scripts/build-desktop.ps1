param([switch]$DebugBuild)
$ErrorActionPreference = 'Stop'
$serviceRoot = Split-Path -Parent $PSScriptRoot
Push-Location $serviceRoot
try {
    Push-Location (Join-Path $serviceRoot 'frontend-panel')
    try {
        & npm run build
        if ($LASTEXITCODE -ne 0) { throw '前端构建失败' }
    } finally { Pop-Location }
    $buildArgs = @('build', '-p', 'teacher_oa_service', '-p', 'teacher_oa_desktop')
    $profileDirectory = 'debug'
    if (-not $DebugBuild) { $buildArgs += '--release'; $profileDirectory = 'release' }
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'Windows 程序构建失败' }
    $destination = Join-Path $serviceRoot 'target/desktop-package'
    New-Item -ItemType Directory -Path $destination -Force | Out-Null
    foreach ($file in @('teacher_oa_service.exe', 'teacher_oa_desktop.exe')) {
        Copy-Item -LiteralPath (Join-Path $serviceRoot "target/$profileDirectory/$file") -Destination $destination -Force
    }
    Copy-Item -LiteralPath (Join-Path $serviceRoot 'config.example.toml') -Destination $destination -Force
    Copy-Item -LiteralPath (Join-Path $serviceRoot 'crates/desktop_manager/README.md') -Destination (Join-Path $destination '桌面管理器说明.md') -Force
    Write-Output "已生成：$destination"
} finally { Pop-Location }
