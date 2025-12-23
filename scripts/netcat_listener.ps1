# Netcat-like listener for testing Myra packet manipulation
# Similar to how clumsy uses ncat for testing

param(
    [Parameter()]
    [int]$Port = 9999,
    
    [Parameter()]
    [ValidateSet("tcp", "udp")]
    [string]$Protocol = "udp",
    
    [Parameter()]
    [switch]$Server,
    
    [Parameter()]
    [switch]$Client,
    
    [Parameter()]
    [string]$Target = "127.0.0.1"
)

Write-Host "=== Myra Network Test Utility ===" -ForegroundColor Cyan
Write-Host ""

# Check if ncat is available (comes with Nmap)
$ncatPath = Get-Command ncat -ErrorAction SilentlyContinue

if (-not $ncatPath) {
    Write-Host "ncat not found. Checking for alternatives..." -ForegroundColor Yellow
    
    # Check for nc (netcat)
    $ncPath = Get-Command nc -ErrorAction SilentlyContinue
    if ($ncPath) {
        $ncatPath = $ncPath
    } else {
        Write-Host ""
        Write-Host "Neither ncat nor nc found on your system." -ForegroundColor Red
        Write-Host ""
        Write-Host "To install ncat, you can:" -ForegroundColor Yellow
        Write-Host "  1. Install Nmap (includes ncat): https://nmap.org/download.html" -ForegroundColor White
        Write-Host "  2. Or use: winget install Insecure.Nmap" -ForegroundColor White
        Write-Host "  3. Or use: choco install nmap" -ForegroundColor White
        Write-Host ""
        exit 1
    }
}

Write-Host "Using: $($ncatPath.Source)" -ForegroundColor Green
Write-Host ""

if ($Server) {
    Write-Host "Starting $Protocol listener on port $Port..." -ForegroundColor Cyan
    Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
    Write-Host ""
    
    if ($Protocol -eq "udp") {
        # UDP server - loop to keep listening after each message
        Write-Host "Filter for Myra: udp.DstPort == $Port or udp.SrcPort == $Port" -ForegroundColor Magenta
        Write-Host ""
        Write-Host "--- Listening for UDP packets ---" -ForegroundColor Green
        Write-Host ""
        
        # Use a loop since UDP ncat doesn't support -k
        while ($true) {
            & $ncatPath.Source -u -l -p $Port -v
        }
    } else {
        # TCP server - keeps listening
        Write-Host "Filter for Myra: tcp.DstPort == $Port or tcp.SrcPort == $Port" -ForegroundColor Magenta
        Write-Host ""
        Write-Host "--- Listening for TCP connections ---" -ForegroundColor Green
        Write-Host ""
        & $ncatPath.Source -l -k -p $Port -v
    }
}
elseif ($Client) {
    Write-Host "Connecting to $Target`:$Port via $Protocol..." -ForegroundColor Cyan
    Write-Host "Type messages and press Enter to send. Press Ctrl+C to stop." -ForegroundColor Yellow
    Write-Host ""
    
    if ($Protocol -eq "udp") {
        & $ncatPath.Source -u $Target $Port -v
    } else {
        & $ncatPath.Source $Target $Port -v
    }
}
else {
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  Start a UDP listener (server):" -ForegroundColor White
    Write-Host "    .\netcat_listener.ps1 -Server -Protocol udp -Port 9999" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  Start a TCP listener (server):" -ForegroundColor White
    Write-Host "    .\netcat_listener.ps1 -Server -Protocol tcp -Port 9999" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  Connect as UDP client:" -ForegroundColor White
    Write-Host "    .\netcat_listener.ps1 -Client -Protocol udp -Target 127.0.0.1 -Port 9999" -ForegroundColor Gray
    Write-Host ""
    Write-Host "  Connect as TCP client:" -ForegroundColor White
    Write-Host "    .\netcat_listener.ps1 -Client -Protocol tcp -Target 127.0.0.1 -Port 9999" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Quick Test Setup:" -ForegroundColor Cyan
    Write-Host "  1. Open two terminals" -ForegroundColor White
    Write-Host "  2. Terminal 1 (server): .\netcat_listener.ps1 -Server -Protocol udp" -ForegroundColor White
    Write-Host "  3. Terminal 2 (client): .\netcat_listener.ps1 -Client -Protocol udp" -ForegroundColor White
    Write-Host "  4. In Myra, use filter: udp.DstPort == 9999 or udp.SrcPort == 9999" -ForegroundColor White
    Write-Host "  5. Enable Freeze module and send messages from client" -ForegroundColor White
    Write-Host ""
}
