# Requires admin privileges - Script must be run as Administrator
# WinDivert Uninstall and Fresh Install Script

# Ensure script is running as Administrator
if (-not ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Host "This script requires Administrator privileges. Please restart as Administrator." -ForegroundColor Red
    exit
}

# Create a temp directory for download
$tempDir = Join-Path $env:TEMP "WinDivertInstaller"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

# Define project directory - change this to your project path if different
$projectDir = $PSScriptRoot
Write-Host "Project directory: $projectDir" -ForegroundColor Cyan

try {
    # STEP 1: Uninstall existing WinDivert
    Write-Host "Uninstalling existing WinDivert..." -ForegroundColor Yellow
    
    # Stop and remove the service if it exists
    Write-Host "Stopping and removing WinDivert service..."
    sc.exe stop WinDivert 2>$null
    sc.exe delete WinDivert 2>$null
    
    # Remove registry entries
    Write-Host "Removing registry entries..."
    Remove-Item -Path "HKLM:\SYSTEM\CurrentControlSet\Services\WinDivert" -Force -Recurse -ErrorAction SilentlyContinue
    
    # Remove driver files
    $driverFiles = @(
        "$env:SystemRoot\System32\drivers\WinDivert.sys",
        "$env:SystemRoot\System32\drivers\WinDivert64.sys",
        "$env:SystemRoot\System32\drivers\WinDivert32.sys"
    )
    
    foreach ($file in $driverFiles) {
        if (Test-Path $file) {
            Write-Host "Removing driver file: $file"
            Remove-Item -Path $file -Force -ErrorAction SilentlyContinue
        }
    }
    
    # Remove DLL files
    $dllFiles = @(
        "$env:SystemRoot\System32\WinDivert.dll",
        "${env:ProgramFiles}\WinDivert\WinDivert.dll",
        "${env:ProgramFiles(x86)}\WinDivert\WinDivert.dll"
    )
    
    foreach ($file in $dllFiles) {
        if (Test-Path $file) {
            Write-Host "Removing DLL file: $file"
            Remove-Item -Path $file -Force -ErrorAction SilentlyContinue
        }
    }
    
    # Cleanup program folders if they exist
    $programFolders = @(
        "${env:ProgramFiles}\WinDivert",
        "${env:ProgramFiles(x86)}\WinDivert"
    )
    
    foreach ($folder in $programFolders) {
        if (Test-Path $folder) {
            Write-Host "Removing folder: $folder"
            Remove-Item -Path $folder -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    
    # Cleanup project directory WinDivert files
    Write-Host "Cleaning up WinDivert files in project directory..." -ForegroundColor Yellow
    
    # Find and remove WinDivert binaries in the project directory
    $projectWinDivertFiles = Get-ChildItem -Path $projectDir -Include "WinDivert*.dll", "WinDivert*.sys" -File -Recurse -ErrorAction SilentlyContinue
    foreach ($file in $projectWinDivertFiles) {
        Write-Host "Removing project file: $($file.FullName)"
        Remove-Item -Path $file.FullName -Force -ErrorAction SilentlyContinue
    }
    
    # Handle specific project structure if known (like src-tauri location)
    $tauriWinDivertPath = Join-Path $projectDir "src-tauri\temp\WinDivert*"
    if (Test-Path $tauriWinDivertPath) {
        Write-Host "Removing Tauri WinDivert temp files..."
        Remove-Item -Path $tauriWinDivertPath -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    Write-Host "Uninstallation completed." -ForegroundColor Green
    
    # STEP 2: Download and install latest WinDivert
    Write-Host "Downloading latest WinDivert..." -ForegroundColor Yellow
    
    # Official WinDivert 2.2.2 download URL
    $windivertUrl = "https://github.com/basil00/Divert/releases/download/v2.2.2/WinDivert-2.2.2-A.zip"
    $zipFile = Join-Path $tempDir "WinDivert.zip"
    $extractPath = Join-Path $tempDir "WinDivert-extract"
    
    # Download the file
    Invoke-WebRequest -Uri $windivertUrl -OutFile $zipFile
    
    # Extract the ZIP file
    Write-Host "Extracting WinDivert files..."
    Expand-Archive -Path $zipFile -DestinationPath $extractPath -Force
    
    # Create installation directory
    $installDir = "$env:ProgramFiles\WinDivert"
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    
    # Copy files to installation directory
    Write-Host "Installing WinDivert to $installDir..."
    
    # Copy appropriate architecture files
    if ([Environment]::Is64BitOperatingSystem) {
        $srcDir = Join-Path $extractPath "WinDivert-2.2.2-A\x64"
    } else {
        $srcDir = Join-Path $extractPath "WinDivert-2.2.2-A\x86"
    }
    
    # Copy all files from the source directory to the installation directory
    Copy-Item -Path "$srcDir\*" -Destination $installDir -Recurse -Force
    
    # Copy include files
    Copy-Item -Path (Join-Path $extractPath "WinDivert-2.2.2-A\include\*") -Destination $installDir -Force
    
    # Copy documentation
    Copy-Item -Path (Join-Path $extractPath "WinDivert-2.2.2-A\doc\*") -Destination $installDir -Force
    
    # Copy driver files to the system directory
    if ([Environment]::Is64BitOperatingSystem) {
        Copy-Item -Path "$installDir\WinDivert64.sys" -Destination "$env:SystemRoot\System32\drivers\" -Force
    } else {
        Copy-Item -Path "$installDir\WinDivert32.sys" -Destination "$env:SystemRoot\System32\drivers\" -Force
    }
    
    # Add installation directory to PATH if not already present
    $envPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
    if ($envPath -notlike "*$installDir*") {
        [Environment]::SetEnvironmentVariable("PATH", "$envPath;$installDir", "Machine")
        Write-Host "Added WinDivert to system PATH."
    }
    
    Write-Host "WinDivert has been successfully installed to $installDir" -ForegroundColor Green
    Write-Host "Installation completed." -ForegroundColor Green
}
catch {
    Write-Host "An error occurred: $_" -ForegroundColor Red
}
finally {
    # Clean up temp files
    if (Test-Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    Write-Host "Temporary files cleaned up." -ForegroundColor Yellow
}

Write-Host "`nWinDivert uninstall and reinstall process completed." -ForegroundColor Green
Write-Host "You may need to restart your computer for all changes to take effect." -ForegroundColor Yellow 