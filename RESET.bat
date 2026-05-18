@echo off
cd /d "%~dp0"
powershell -ExecutionPolicy Bypass -File ".\scripts\full-reset.ps1"
pause
