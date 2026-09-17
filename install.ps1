# Install prebuilt tkt from GitHub Releases into %LOCALAPPDATA%\tkt (or $Env:BIN_DIR).
# Usage: irm https://raw.githubusercontent.com/Comninos/tkt/master/install.ps1 | iex

# Wrap in a function so `irm | iex` does not bind pipeline input to script statements.
function Install-Tkt {
    $ErrorActionPreference = "Stop"

    $Repo = if ($Env:TKT_REPO) { $Env:TKT_REPO } else { "Comninos/tkt" }
    $BinName = "tkt.exe"

    $localAppData = $Env:LOCALAPPDATA
    if ([string]::IsNullOrWhiteSpace($localAppData)) {
        $localAppData = [Environment]::GetFolderPath("LocalApplicationData")
    }
    if ([string]::IsNullOrWhiteSpace($localAppData)) {
        throw "LOCALAPPDATA is not set; set BIN_DIR to an install directory and retry"
    }

    $BinDir = if ($Env:BIN_DIR) { $Env:BIN_DIR } else { Join-Path $localAppData "tkt" }

    # Prefer PROCESSOR_* env vars. On Windows PowerShell 5.1,
    # [RuntimeInformation]::OSArchitecture can resolve the wrong assembly and be null.
    $arch = $Env:PROCESSOR_ARCHITECTURE
    if (-not [string]::IsNullOrWhiteSpace($Env:PROCESSOR_ARCHITEW6432)) {
        $arch = $Env:PROCESSOR_ARCHITEW6432
    }
    if ([string]::IsNullOrWhiteSpace($arch)) {
        if ([Environment]::Is64BitOperatingSystem) { $arch = "AMD64" }
        else { $arch = "x86" }
    }

    switch ($arch.ToUpperInvariant()) {
        "AMD64" { $target = "x86_64-pc-windows-msvc" }
        "ARM64" { $target = "aarch64-pc-windows-msvc" }
        "X86" { throw "32-bit Windows is not supported" }
        default { throw "unsupported architecture: $arch" }
    }

    $asset = "tkt-$target.zip"
    $url = "https://github.com/$Repo/releases/latest/download/$asset"

    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("tkt-install-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $tmp | Out-Null

    try {
        $zipPath = Join-Path $tmp $asset
        Write-Host "downloading $asset..."
        try {
            Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        } catch {
            throw "no prebuilt binary for $target. Build from source with: cargo install --git https://github.com/$Repo --locked"
        }

        Expand-Archive -Path $zipPath -DestinationPath $tmp -Force
        $src = Get-ChildItem -Path $tmp -Filter $BinName -Recurse -File | Select-Object -First 1
        if (-not $src) {
            throw "archive did not contain $BinName"
        }

        New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
        $dest = Join-Path $BinDir $BinName
        Copy-Item -Path $src.FullName -Destination $dest -Force

        $pathEntries = @()
        if (-not [string]::IsNullOrWhiteSpace($Env:Path)) {
            $pathEntries = @($Env:Path -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
        }
        if ($pathEntries -notcontains $BinDir) {
            $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
            if ($null -eq $userPath) { $userPath = "" }
            $userEntries = @($userPath -split ";" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
            if ($userEntries -notcontains $BinDir) {
                $newPath = if ($userPath.Trim().Length -gt 0) { "$userPath;$BinDir" } else { $BinDir }
                [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
                $Env:Path = if ([string]::IsNullOrWhiteSpace($Env:Path)) { $BinDir } else { "$Env:Path;$BinDir" }
                Write-Host "added $BinDir to your user PATH (restart the shell if needed)"
            }
        }

        Write-Host "installed: $dest"
    } finally {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}

Install-Tkt
