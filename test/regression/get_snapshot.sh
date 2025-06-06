#!/bin/bash

# Usage: ./get_snapshot.sh <model_dir>
#
# Example: ./get_snapshot.sh ~/Projects/veery/model
#
# Run this script to get a snapshot of the current output of the interpreter for
# `radar.on` and `scatterometer.on`. This script assumes that the model directory
# contains all files needed to run `radar.on` and `scatterometer.on`
#
# If the snapshot is acceptable, rename it to <filename>_<branch>_snapshot.out and commit
# it as the 'reference' snapshot for that model.

current_dir=$(pwd)
model_dir="$1"

if [ -z "$model_dir" ]; then
    echo "Error: Model directory not provided"
    echo "Usage: $0 <model_dir>"
    exit 1
fi

# Ensure snapshots directory exists
mkdir -p "${current_dir}/snapshots"

cd "$model_dir"

# Get a snapshot of the current output of the interpreter for a given file on a given branch
get_snapshot() {
    local file="$1"
    local branch="$2"

    local original_branch=$(git branch --show-current)

    echo -n "Getting snapshot for '${file}' on branch '${branch}' ... "

    git checkout "$branch" > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo "failed to checkout branch"
        return 1
    fi

    if [ ! -f "$file" ]; then
        echo "file not found"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    # Set up output file path
    local file_commit_hash=$(git rev-parse --short=16 "$branch")
    local output_file="${current_dir}/snapshots/${file%.on}_${branch}_${file_commit_hash}.out"

    # Create or clear the output file
    > "$output_file"

    # Generate header
    echo "Run on commit ${file_commit_hash} of file ${file} (branch: ${branch})" >> "$output_file"
    echo "--------------------------------" >> "$output_file"

    # Run oneil and capture output
    if yes "quit" | oneil "$file" tree summarize all dependents independent test reload >> "$output_file" 2>&1; then
        echo "done"
        echo "Output saved to: ${output_file}"
        echo "If acceptable, rename to: ${current_dir}/snapshots/${file%.on}_${branch}_snapshot.out"
    else
        echo "failed to run oneil"
        rm "$output_file"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    git checkout "$original_branch" > /dev/null 2>&1
    return 0
}

# Track overall success/failure
exit_code=0

# Get snapshots for each file
get_snapshot "radar.on" "0.5.0" || exit_code=1
get_snapshot "scatterometer.on" "1.0.0" || exit_code=1

exit $exit_code