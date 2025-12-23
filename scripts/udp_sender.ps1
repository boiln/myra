# Continuous UDP packet sender for testing Myra
# Sends numbered packets every 500ms like clumsy demo

param(
    [int]$Port = 9999,
    [string]$Target = "127.0.0.1",
    [int]$IntervalMs = 500
)

Write-Host "=== Myra UDP Packet Sender ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Sending UDP packets to $Target`:$Port every ${IntervalMs}ms" -ForegroundColor Yellow
Write-Host "Press Ctrl+C to stop" -ForegroundColor Yellow
Write-Host ""
Write-Host "Myra Filter: outbound and udp.DstPort == $Port" -ForegroundColor Magenta
Write-Host ""

$udpClient = New-Object System.Net.Sockets.UdpClient
$endpoint = New-Object System.Net.IPEndPoint([System.Net.IPAddress]::Parse($Target), $Port)

$counter = 1

try {
    while ($true) {
        $message = "$counter"
        $bytes = [System.Text.Encoding]::ASCII.GetBytes($message + "`n")
        
        $udpClient.Send($bytes, $bytes.Length, $endpoint) | Out-Null
        
        Write-Host $counter
        
        $counter++
        Start-Sleep -Milliseconds $IntervalMs
    }
}
finally {
    $udpClient.Close()
}
