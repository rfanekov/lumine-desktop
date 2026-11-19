@echo off
setlocal

set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VSPATH="
set "VSDEVCMD="

if exist "%VSWHERE%" (
  for /f "usebackq delims=" %%i in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath`) do set "VSPATH=%%i"
)

if defined VSPATH (
  if exist "%VSPATH%\Common7\Tools\VsDevCmd.bat" set "VSDEVCMD=%VSPATH%\Common7\Tools\VsDevCmd.bat"
)

if defined VSDEVCMD (
  call "%VSDEVCMD%" -arch=x64 -host_arch=x64 -no_logo >nul
) else (
  echo [lumine] Visual Studio C++ tools not found, using current environment 1>&2
)

%*
exit /b %ERRORLEVEL%
