#!/bin/bash

# Usage: ./check_snapshot.sh <model_dir>
#
# Example: ./check_snapshot.sh ~/Projects/veery/model
#
# Run this script to check if the current output of the interpreter for
# `radar.on` and `scatterometer.on` matches their reference snapshots.
# This script assumes that the model directory contains all files needed
# to run `radar.on` and `scatterometer.on`, and that reference snapshots
# exist in the snapshots directory.

current_dir=$(pwd)
model_dir="$1"

if [ -z "$model_dir" ]; then
    echo "Error: Model directory not provided"
    echo "Usage: $0 <model_dir>"
    exit 1
fi

cd "$model_dir"

# Get the current git commit hash
commit_hash=$(git rev-parse --short=16 HEAD)

# Check snapshot for a given file on a given branch
check_snapshot() {
    local file="$1"
    local branch="$2"

    local original_branch=$(git branch --show-current)
    
    echo -n "Checking snapshot for '${file}' on branch '${branch}' ... "

    git checkout "$branch" > /dev/null 2>&1

    if [ ! -f "$file" ]; then
        echo "file not found"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    # Create temporary file for current output
    local file_commit_hash=$(git rev-parse --short=16 "$branch")
    local temp_output=$(mktemp)
    
    # Generate header for temp file
    echo "Run on commit ${file_commit_hash} of file ${file} (branch: ${branch})" > "$temp_output"
    echo "--------------------------------" >> "$temp_output"
    
    # Run oneil and capture output
    yes "quit" | oneil "$file" tree summarize all dependents independent test reload >> "$temp_output"

    # Get reference snapshot file
    local ref_snapshot="${current_dir}/snapshots/${file%.on}_${branch}_snapshot.out"

    if [ ! -f "$ref_snapshot" ]; then
        echo "reference snapshot not found"
        rm "$temp_output"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    # Compare outputs, ignoring the commit hash line
    if diff "$temp_output" "$ref_snapshot" > /dev/null; then
        echo "matches reference"
        rm "$temp_output"
        git checkout "$original_branch" > /dev/null 2>&1
        return 0
    else
        local diff_file="${current_dir}/snapshots/${file%.on}_${branch}_${file_commit_hash}.out"
        mv "$temp_output" "$diff_file"

        echo "MISMATCH detected"
        echo "Differences found between current output and reference snapshot."
        echo "Check differences with:"
        echo "diff $diff_file $ref_snapshot"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi
}

# Track overall success/failure
exit_code=0

# Check snapshots for each file
check_snapshot "radar.on" "0.5.0" || exit_code=1
check_snapshot "scatterometer.on" "1.0.0" || exit_code=1

exit $exit_code 