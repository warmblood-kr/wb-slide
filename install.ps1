# wb-slide installer for Windows PowerShell
# Usage:
#   irm https://raw.githubusercontent.com/warmblood-kr/wb-slide/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "warmblood-kr/wb-slide"
$Binary = "wb-slide.exe"

function Get-InstallDir {
    if ($env:WB_SLIDE_INSTALL_DIR) {
        return $env:WB_SLIDE_INSTALL_DIR
    }
    # Default: user-owned location, no admin needed
    return "$env:LOCALAPPDATA\Programs\wb-slide"
}

function Get-LatestVersion {
    $response = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
    return $response.tag_name
}

function Get-Platform {
    # Try multiple methods for arch detection; older PowerShell may lack one.
    $arch = $null
    try {
        $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    } catch { }
    if (-not $arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
    if (-not $arch) { $arch = $env:PROCESSOR_ARCHITEW6432 }

    $archUpper = ($arch | Out-String).Trim().ToUpper()
    switch -Regex ($archUpper) {
        '^(X64|AMD64)$'  { return "windows-x64" }
        '^(ARM64)$'      { Write-Error "Windows ARM64 not yet supported"; exit 1 }
        default {
            Write-Error "Unsupported architecture: '$archUpper' (only x64 supported on Windows)"
            exit 1
        }
    }
}

function Test-PathContains($Dir) {
    $paths = $env:PATH -split ';'
    return $paths -contains $Dir
}

function Main {
    $version = if ($args.Count -gt 0) { $args[0] } else { Get-LatestVersion }
    if (-not $version) {
        Write-Error "Could not determine latest version"
        exit 1
    }

    $platform = Get-Platform
    $installDir = Get-InstallDir

    Write-Host "Installing wb-slide $version for $platform..."
    Write-Host "Target: $installDir"
    Write-Host ""

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null

    $asset = "wb-slide-$platform.zip"
    $url = "https://github.com/$Repo/releases/download/$version/$asset"
    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "wb-slide-install"
    if (Test-Path $tmpDir) { Remove-Item -Recurse -Force $tmpDir }
    New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

    $zipPath = Join-Path $tmpDir $asset
    Invoke-WebRequest -Uri $url -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

    $src = Join-Path $tmpDir $Binary
    $dst = Join-Path $installDir $Binary
    Move-Item -Force -Path $src -Destination $dst

    Remove-Item -Recurse -Force $tmpDir

    Write-Host "Installed: $dst"
    Write-Host ""

    if (-not (Test-PathContains $installDir)) {
        Write-Host "WARNING: $installDir is not in your PATH." -ForegroundColor Yellow
        Write-Host "Add it for the current user with:"
        Write-Host ""
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$installDir', 'User')"
        Write-Host ""
        Write-Host "Then restart your terminal."
        Write-Host ""
    }

    Write-Host "Run 'wb-slide show' in a directory with slides.md to start presenting."
}

Main @args
