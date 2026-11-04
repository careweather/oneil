@echo off
setlocal EnableExtensions

rem Directory containing this script (trailing backslash)
set "SCRIPT_DIR=%~dp0"
set "WITH_PYTHON_COMPILER=0"
set "EDITABLE=0"

:arg_loop
if "%~1"=="" goto arg_done
if /i "%~1"=="--with-python-compiler" (
  set "WITH_PYTHON_COMPILER=1"
  shift
  goto arg_loop
)
if /i "%~1"=="--with-python-package" (
  set "WITH_PYTHON_COMPILER=1"
  shift
  goto arg_loop
)
if /i "%~1"=="--no-python" (
  echo Note: --no-python skips the pip Python library; the CLI still includes python-lib. 1>&2
  shift
  goto arg_loop
)
if /i "%~1"=="-e" (
  set "EDITABLE=1"
  shift
  goto arg_loop
)
if /i "%~1"=="--editable" (
  set "EDITABLE=1"
  shift
  goto arg_loop
)
if /i "%~1"=="-h" goto show_help
if /i "%~1"=="--help" goto show_help
echo Unknown option: %~1 1>&2
call :usage 1>&2
exit /b 1

:show_help
call :usage
exit /b 0

:arg_done
if "%EDITABLE%"=="1" if not "%WITH_PYTHON_COMPILER%"=="1" (
  echo Note: --editable only applies with --with-python-package. 1>&2
)

where cargo >nul 2>&1
if errorlevel 1 (
  echo Error: Cargo was not found on your PATH. 1>&2
  echo. 1>&2
  echo Install the Rust toolchain with rustup: 1>&2
  echo   https://rustup.rs/ 1>&2
  echo. 1>&2
  echo Download and run rustup-init.exe from the site above. 1>&2
  echo. 1>&2
  echo Then restart this terminal, or ensure %%USERPROFILE%%\.cargo\bin is on your PATH. 1>&2
  exit /b 1
)

call :require_c_toolchain
if errorlevel 1 exit /b 1

set "ONEIL_PKG=%SCRIPT_DIR%src\oneil"
if not exist "%ONEIL_PKG%\Cargo.toml" (
  echo Error: expected Cargo.toml at "%ONEIL_PKG%" 1>&2
  exit /b 1
)

set "PYTHON_CMD="
call :try_uv 3.14
if not defined PYTHON_CMD call :try_uv 3.12
if not defined PYTHON_CMD call :try_on_path python3.14
if not defined PYTHON_CMD call :try_on_path python3.12
if not defined PYTHON_CMD (
  where python3 >nul 2>&1
  if not errorlevel 1 set "PYTHON_CMD=python3"
)
if not defined PYTHON_CMD (
  where python >nul 2>&1
  if not errorlevel 1 set "PYTHON_CMD=python"
)
if not defined PYTHON_CMD (
  echo Error: Python 3.12 or 3.14 was not found ^(needed to link the Rust CLI's model Python support^). 1>&2
  echo. 1>&2
  echo Install it with: uv python install 3.14 1>&2
  exit /b 1
)

set "PYO3_PYTHON=%PYTHON_CMD%"

%PYTHON_CMD% -c "import sys; sys.exit(0 if sys.version_info[:2] in ((3, 12), (3, 14)) else 1)" >nul 2>&1
if errorlevel 1 (
  for /f "usebackq delims=" %%V in (`%PYTHON_CMD% --version 2^>^&1`) do echo Error: Oneil supports Python 3.12 and 3.14. Found: %%V 1>&2
  echo Install one with: uv python install 3.14 1>&2
  exit /b 1
)

%PYTHON_CMD% -c "import os, sys, sysconfig; inc=sysconfig.get_path('include'); sys.exit(0 if os.path.isfile(os.path.join(inc, 'Python.h')) else 1)" >nul 2>&1
if errorlevel 1 (
  echo Error: Python development headers were not found ^(Python.h is missing^). 1>&2
  echo. 1>&2
  echo The Rust CLI links against Python so models can import .py files. 1>&2
  echo. 1>&2
  echo On Windows, use the python.org installer and enable optional features, or install matching 1>&2
  echo debug/header packages for your Python distribution, then re-run this script. 1>&2
  exit /b 1
)

if "%WITH_PYTHON_COMPILER%"=="1" if not exist "%SCRIPT_DIR%pyproject.toml" (
  echo Error: pyproject.toml not found at "%SCRIPT_DIR%" 1>&2
  exit /b 1
)

echo Installing oneil and oneil-runner ^(default features: rust-lib + python-lib^)...
cargo install --force --path "%SCRIPT_DIR%src\oneil_loader"
if errorlevel 1 exit /b 1
cargo install --force --path "%ONEIL_PKG%"
if errorlevel 1 exit /b 1

if not "%WITH_PYTHON_COMPILER%"=="1" goto finish

echo Installing Python library ^(import oneil^)...
pushd "%SCRIPT_DIR%"
if "%EDITABLE%"=="1" (
  %PYTHON_CMD% -m pip install -e .
) else (
  %PYTHON_CMD% -m pip install .
)
if errorlevel 1 (
  popd
  exit /b 1
)
popd

:finish
echo.
echo Done.
echo Ensure %%USERPROFILE%%\.cargo\bin is on your PATH to run: oneil --version
exit /b 0

:require_c_toolchain
where gcc >nul 2>&1
if not errorlevel 1 exit /b 0
rustc -vV 2>nul | findstr /i /c:"windows-msvc" >nul
if not errorlevel 1 exit /b 0
echo Error: gcc was not found on your PATH. 1>&2
echo. 1>&2
echo A C compiler is required to build Oneil ^(Rust linking and native extensions^). 1>&2
echo. 1>&2
echo Install one of: 1>&2
echo   - MSYS2/MinGW-w64: https://www.msys2.org/ ^(e.g. pacman -S mingw-w64-ucrt-x86_64-gcc^) 1>&2
echo   - Visual Studio Build Tools with the "Desktop development with C++" workload ^(MSVC^); 1>&2
echo     use the MSVC Rust target ^(default rustup on Windows^) so Cargo can link without gcc. 1>&2
exit /b 1

:usage
echo Usage: install.bat [options]
echo.
echo   Builds and installs the Rust Oneil CLI with Cargo ^(rust-lib + python-lib^).
echo   That build links PyO3 so models can import ordinary .py files and helper
echo   modules can import oneil.
echo.
echo Options:
echo   --with-python-package   Also install the Python library via pip.
echo   -e, --editable          With --with-python-package, install it editable.
echo   -h, --help              Show this help.
echo.
echo Prerequisites:
echo   - Cargo ^(Rust^): https://rustup.rs/
echo   - gcc ^(or MSVC with the windows-msvc Rust target; see error text if checks fail^)
echo   - Python 3.12 or 3.14 development headers ^(uv python install 3.14^)
echo   - For --with-python-package: that same Python with pip
goto :eof

:try_uv
where uv >nul 2>&1
if errorlevel 1 exit /b 0
for /f "usebackq delims=" %%P in (`uv python find %~1 2^>nul`) do set "PYTHON_CMD=%%P"
exit /b 0

:try_on_path
where %~1 >nul 2>&1
if errorlevel 1 exit /b 0
set "PYTHON_CMD=%~1"
exit /b 0
