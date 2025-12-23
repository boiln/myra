# Start the ncat server in a new PowerShell window
Start-Process powershell -ArgumentList "-NoExit -Command `"ncat -u -l 9911`""

# Wait a moment for the server to start
Start-Sleep -Seconds 2

# Start the Python script in another PowerShell window
$scriptPath = Join-Path $PSScriptRoot "send_udp_nums.py"
Start-Process powershell -ArgumentList "-NoExit -Command `"python '$scriptPath' --nosleep localhost 9911`""
