cd /d "%~dp0"

taskkill /F /IM udpdeminer-x86_64-pc-windows-gnu.exe
taskkill /F /IM hysteria-windows-amd64.exe
start udpdeminer-x86_64-pc-windows-gnu.exe -b 0.0.0.0 -s 1.2.3.4 -p 5678 --outbound 0.0.0.0 --hookpath ./ntfyn4hook.bat --hookip customize
start hysteria-windows-amd64.exe client -c client.yaml --disable-update-check