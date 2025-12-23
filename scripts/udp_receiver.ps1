# UDP receiver for testing Myra
# Displays received packets with timestamps

param(
    [int]$Port = 9999
)

Write-Host "=== Myra UDP Packet Receiver ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Listening on UDP port $Port..." -ForegroundColor Yellow
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""
Write-Host "Myra Filter: inbound and udp.DstPort == $Port" -ForegroundColor Magenta
Write-Host ""
Write-Host "--- Waiting for packets ---" -ForegroundColor Green
Write-Host ""

$udpClient = New-Object System.Net.Sockets.UdpClient($Port)
$endpoint = New-Object System.Net.IPEndPoint([System.Net.IPAddress]::Any, 0)

try {
    while ($true) {
        $bytes = $udpClient.Receive([ref]$endpoint)
        $message = [System.Text.Encoding]::ASCII.GetString($bytes).Trim()
        
        Write-Host $message
    }
}
finally {
    $udpClient.Close()
}
