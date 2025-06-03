@echo off
cd /d "%~dp0"

set EVENT=%1
set STREAMID=%2
set SRVSTR=%3
set LSTR=%4
set TOPICURL="ntfy.sh/link2lantest"

if "%EVENT%"=="getsrvstr" (
	link2lan-x86_64-pc-windows-gnu.exe --plan 104 --topicurl %TOPICURL% --mynattype 3 --event %EVENT% --streamid %STREAMID% --srvstr %SRVSTR% --localstr %LSTR%
)
