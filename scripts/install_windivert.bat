@echo off
echo Running WinDivert uninstall and reinstall script with administrator privileges...
powershell -Command "Start-Process powershell -ArgumentList '-ExecutionPolicy Bypass -File \"%~dp0windivert.ps1\"' -Verb RunAs"
echo If a UAC prompt appeared, please accept it to continue.
echo.
pause