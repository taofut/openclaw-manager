@echo off
setlocal enabledelayedexpansion

:: ─── Require Administrator privileges ────────────────────────────────────────
net session >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Requesting administrator privileges...
    powershell -Command "Start-Process cmd.exe -ArgumentList '/c \"%~f0\"' -Verb RunAs -Wait"
    exit /b 0
)

echo ========================================
echo          Git Environment Setup
echo ========================================
echo.

set "GIT_INSTALLER=Git-2.53.0-64-bit.exe"
set "GIT_URL=http://192.168.18.77:12080/%GIT_INSTALLER%"
set "GIT_INSTALL_DIR=D:\Git"

:: ─── Check Git ───────────────────────────────────────────────────────────────
where git >nul 2>&1
if not "!errorlevel!"=="0" (
    echo [INFO] Git not detected. Starting automatic installation...
    echo.

    :: Download installer to current script directory
    echo Downloading %GIT_INSTALLER% ...
    curl -L --progress-bar -o "%~dp0%GIT_INSTALLER%" "%GIT_URL%"
    if not exist "%~dp0%GIT_INSTALLER%" (
        echo [ERROR] Download failed. Please install manually: %GIT_URL%
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Download complete: %~dp0%GIT_INSTALLER%
    echo.

    :: Silent install to D:\Git
    echo Installing Git to %GIT_INSTALL_DIR% ...
    "%~dp0%GIT_INSTALLER%" /VERYSILENT /NORESTART /NOCANCEL /SP- /CLOSEAPPLICATIONS /RESTARTAPPLICATIONS /DIR="%GIT_INSTALL_DIR%"
    set "inst_exit=!errorlevel!"
    if not "!inst_exit!"=="0" (
        echo [ERROR] Installation failed ^(exit code: !inst_exit!^).
        if not "%SKIP_PAUSE%"=="1" pause
        exit /b 1
    )
    echo [OK] Git installed to %GIT_INSTALL_DIR%
    echo.

    :: Make git available in the current session
    set "PATH=%GIT_INSTALL_DIR%\bin;!PATH!"
    goto :configure_git
) else (
    echo [OK] Git detected
    echo      Version:
    git --version
    echo.
    goto :configure_git
)

:configure_git
:: Configure GitHub HTTPS redirect
echo Configuring GitHub HTTPS redirect...
git.exe config --global url."https://github.com/".insteadOf git@github.com:
for /f "delims=" %%V in ('git.exe config --global --get url."https://github.com/".insteadOf 2^>nul') do set "gitval=%%V"
if "!gitval!"=="git@github.com:" (
    echo [OK] GitHub HTTPS redirect configured successfully
    echo      Current value: !gitval!
) else (
    echo [ERROR] Failed to configure GitHub HTTPS redirect, current value: !gitval!
    if not "%SKIP_PAUSE%"=="1" pause
)
echo.

:: Configure git:// protocol HTTPS redirect
echo Configuring git:// protocol HTTPS redirect...
git.exe config --global url."https://".insteadOf git://
for /f "delims=" %%V in ('git.exe config --global --get url."https://".insteadOf 2^>nul') do set "gitval=%%V"
if "!gitval!"=="git://" (
    echo [OK] git:// protocol HTTPS redirect configured successfully
    echo      Current value: !gitval!
) else (
    echo [ERROR] Failed to configure git:// protocol HTTPS redirect, current value: !gitval!
    if not "%SKIP_PAUSE%"=="1" pause
)
echo.

echo ========================================
echo Setup Complete
echo ========================================
echo.
if "%SKIP_PAUSE%"=="1" (
    exit /b 0
)
echo Press any key to exit...
pause >nul
exit /b 0
