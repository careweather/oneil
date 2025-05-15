@echo off

REM Get the directory of the script
SET "SCRIPT_DIR=%~dp0"

REM Initialize variables
SET "EDITABLE=false"

REM Parse options
FOR %%A IN (%*) DO (
    IF "%%A"=="-e" (
        SET "EDITABLE=true"
    )
)

REM Install dependencies using absolute paths
IF EXIST "%SCRIPT_DIR%src\oneil\requirements.txt" (
    pip install -r "%SCRIPT_DIR%src\oneil\requirements.txt"
) ELSE (
    ECHO ERROR: requirements.txt not found at "%SCRIPT_DIR%src\oneil\requirements.txt"
    EXIT /B 1
)

REM Install package
IF "%EDITABLE%"=="true" (
    pip install -e "%SCRIPT_DIR%"
) ELSE (
    pip install %SCRIPT_DIR%
)

REM Check if Vim is installed
where vim >nul 2>&1
IF ERRORLEVEL 1 (
    ECHO Vim not found, installing...
    choco install vim -y
) ELSE (
    ECHO Vim is already installed.
)

REM Set up Vim syntax highlighting
SET "VIM_DIR=%USERPROFILE%\vimfiles"
SET "VIM_SYNTAX_DIR=%VIM_DIR%\syntax"
SET "VIM_FTDETECT_DIR=%VIM_DIR%\ftdetect"
SET "ONEIL_VIM_DIR=%SCRIPT_DIR%vim"

REM Create Vim directories if they do not exist
IF NOT EXIST "%VIM_DIR%" mkdir "%VIM_DIR%"
IF NOT EXIST "%VIM_SYNTAX_DIR%" mkdir "%VIM_SYNTAX_DIR%"
IF NOT EXIST "%VIM_FTDETECT_DIR%" mkdir "%VIM_FTDETECT_DIR%"

REM Copy syntax and ftdetect files
IF EXIST "%ONEIL_VIM_DIR%\syntax\oneil.vim" (
    copy /Y "%ONEIL_VIM_DIR%\syntax\oneil.vim" "%VIM_SYNTAX_DIR%\oneil.vim"
) ELSE (
    ECHO ERROR: Syntax file not found at "%ONEIL_VIM_DIR%\syntax\oneil.vim"
    EXIT /B 1
)

IF EXIST "%ONEIL_VIM_DIR%\ftdetect\oneil.vim" (
    copy /Y "%ONEIL_VIM_DIR%\ftdetect\oneil.vim" "%VIM_FTDETECT_DIR%\oneil.vim"
) ELSE (
    ECHO ERROR: File detection file not found at "%ONEIL_VIM_DIR%\ftdetect\oneil.vim"
    EXIT /B 1
)

ECHO Vim syntax highlighting setup completed.
