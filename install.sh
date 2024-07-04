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

