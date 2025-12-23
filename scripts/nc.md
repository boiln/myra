# Terminal 1 - Start server

~\workflow\myra-tauri\scripts\netcat_listener.ps1 -Server -Protocol udp -Port 9999

# Terminal 2 - Start client

~\workflow\myra-tauri\scripts\netcat_listener.ps1 -Client -Protocol udp -Port 9999

~\workflow\myra-tauri\scripts\udp_receiver.ps1

~\workflow\myra-tauri\scripts\udp_sender.ps1

ncat -u -l -p 9999

python C:\Users\vR\workflow\myra-tauri\scripts\send_udp_nums.py 127.0.0.1 9999
