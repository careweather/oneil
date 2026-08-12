@echo off
setlocal EnableExtensions

rem Directory containing this script (trailing backslash)
set "SCRIPT_DIR=%~dp0"
set "NO_PYTHON=0"
set "EDITABLE=0"

:arg_loop
if "%~1"=="" goto arg_done
if /i "%~1"=="--no-python" (
  set "NO_PYTHON=1"
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
if "%NO_PYTHON%"=="1" if "%EDITABLE%"=="1" (
  echo Note: --editable has no effect with --no-python. 1>&2
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

set "ONEIL_PKG=%SCRIPT_DIR%src-rs\oneil"
if not exist "%ONEIL_PKG%\Cargo.toml" (
  echo Error: expected Cargo.toml at "%ONEIL_PKG%" 1>&2
  exit /b 1
)

if "%NO_PYTHON%"=="1" goto do_cargo_install

if not exist "%SCRIPT_DIR%pyproject.toml" (
  echo Error: pyproject.toml not found at "%SCRIPT_DIR%" 1>&2
  exit /b 1
)

set "PYTHON_CMD="
where python3 >nul 2>&1
if not errorlevel 1 set "PYTHON_CMD=python3"
if not defined PYTHON_CMD (
  where python >nul 2>&1
  if not errorlevel 1 set "PYTHON_CMD=python"
)
if not defined PYTHON_CMD (
  echo Error: Python 3.10+ is required for the library install but no python3/python was found. 1>&2
  echo. 1>&2
  echo Install Python, or re-run with --no-python to install only the CLI with no Python bindings. 1>&2
  exit /b 1
)

%PYTHON_CMD% -c "import sys; sys.exit(0 if sys.version_info >= (3, 10) else 1)" >nul 2>&1
if errorlevel 1 (
  for /f "usebackq delims=" %%V in (`%PYTHON_CMD% --version 2^>^&1`) do echo Error: Python 3.10 or newer is required. Found: %%V 1>&2
  exit /b 1
)

%PYTHON_CMD% -c "import os, sys, sysconfig; inc=sysconfig.get_path('include'); sys.exit(0 if os.path.isfile(os.path.join(inc, 'Python.h')) else 1)" >nul 2>&1
if errorlevel 1 (
  echo Error: Python development headers were not found ^(Python.h is missing^). 1>&2
  echo. 1>&2
  echo The CLI build links against Python; install headers before building. 1>&2
  echo. 1>&2
  echo On Windows, use the python.org installer and enable optional features, or install matching 1>&2
  echo debug/header packages for your Python distribution, then re-run this script. 1>&2
  exit /b 1
)

:do_cargo_install
echo Installing Oneil CLI with Cargo...
cargo install --force --path "%ONEIL_PKG%" --no-default-features --features rust-lib
if errorlevel 1 exit /b 1

if "%NO_PYTHON%"=="1" goto finish

echo Installing Oneil Python package (deprecated)...
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
echo   Builds and installs the Oneil CLI with Cargo. By default, also installs the
echo   Python package ^(import oneil^) for the current interpreter.
echo.
echo Options:
echo   --no-python    Install the CLI only: no Python bindings and no pip package.
echo   -e, --editable Install the Python package in editable mode ^(development^).
echo   -h, --help     Show this help.
echo.
echo Prerequisites:
echo   - Cargo ^(Rust^): https://rustup.rs/
echo   - gcc ^(or MSVC with the windows-msvc Rust target; see error text if checks fail^)
echo   - For the default install: Python 3.10+ with pip ^(python3 -m pip / python -m pip^)
echo     and Python development headers ^(Python.h^)
goto :eof
