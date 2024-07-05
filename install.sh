#!/bin/bash

# Get the directory of the script
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Initialize variables
EDITABLE=false

# Parse options
while getopts "e" opt; do
  case ${opt} in
    e )
      EDITABLE=true
      ;;
    \? )
      echo "Usage: cmd [-e]"
      exit 1
      ;;
  esac
done

# Install dependencies using absolute paths
pip3 install -r "$SCRIPT_DIR/src/oneil/requirements.txt"

# Install package
if [ "$EDITABLE" = true ]; then
  pip3 install -e "$SCRIPT_DIR"
else
  pip3 install "$SCRIPT_DIR"
fi

# Check if Vim is installed, install if not
if ! command -v vim &> /dev/null; then
  echo "Vim not found, installing..."
  sudo apt-get update
  sudo apt-get install -y vim
else
  echo "Vim is already installed."
fi

# Set up Vim syntax highlighting
VIM_DIR=~/.vim
VIM_SYNTAX_DIR=$VIM_DIR/syntax
VIM_FTDETECT_DIR=$VIM_DIR/ftdetect
ONEIL_VIM_DIR="$SCRIPT_DIR/vim"

# Create Vim directories if they do not exist
mkdir -p $VIM_DIR
mkdir -p $VIM_SYNTAX_DIR
mkdir -p $VIM_FTDETECT_DIR

# Create symbolic links for syntax and ftdetect files
ln -sf "$ONEIL_VIM_DIR/syntax/oneil.vim" "$VIM_SYNTAX_DIR/oneil.vim"
ln -sf "$ONEIL_VIM_DIR/ftdetect/oneil.vim" "$VIM_FTDETECT_DIR/oneil.vim"

echo "Vim syntax highlighting setup completed."

