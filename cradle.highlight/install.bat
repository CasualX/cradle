@echo off
setlocal

set "SOURCE_DIR=%~dp0"
if "%SOURCE_DIR:~-1%"=="\" set "SOURCE_DIR=%SOURCE_DIR:~0,-1%"

set "TARGET_ROOT=%USERPROFILE%\.vscode\extensions"
set "TARGET_DIR=%TARGET_ROOT%\cradle.highlight"

if not exist "%TARGET_ROOT%" (
	echo VS Code extensions directory not found: %TARGET_ROOT%
	exit /b 1
)

if exist "%TARGET_DIR%" rmdir /S /Q "%TARGET_DIR%"
mkdir "%TARGET_DIR%"

xcopy "%SOURCE_DIR%\*" "%TARGET_DIR%\" /E /I /Y >nul
if errorlevel 1 (
	echo Failed to install cradle.highlight.
	exit /b 1
)

echo Installed cradle.highlight to %TARGET_DIR%
exit /b 0
