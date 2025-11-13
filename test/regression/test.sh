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

# script colors
error_start="\e[31m"
info_start="\e[34m"
success_start="\e[32m"
reset_color="\e[0m"
error_string="[${error_start}ERROR${reset_color}]"
info_string="[${info_start}INFO ${reset_color}]"

oneil_commit=$(git rev-parse --short=8 HEAD)
current_dir=$(pwd)
model_dir="$1"

if [ -z "$model_dir" ]; then
    echo -e "$error_string model directory not provided"
    echo -e "Usage: $0 <model_dir>"
    exit 1
fi

cd "$model_dir"

# Check snapshot for a given file on a given commit
check_snapshot() {
    local file="$1"
    local veery_commit="$2"

    local original_branch=$(git branch --show-current)
    
    echo -ne "$info_string checking snapshot for '${file}' on commit '${veery_commit}' ... "

    git checkout "$veery_commit" > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo -e "${error_start}failed to checkout commit${reset_color}"
        return 1
    fi

    if [ ! -f "$file" ]; then
        echo -e "${error_start}file not found${reset_color}"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    # Create temporary file for current output
    local temp_output=$(mktemp)
    
    # Generate header for temp file
    echo "Run on commit ${veery_commit} of file ${file}" > "$temp_output"
    echo "--------------------------------" >> "$temp_output"
    
    # Run oneil and capture output
    yes "quit" | oneil regression-test "$file" >> "$temp_output"

    # Get reference snapshot file
    local ref_snapshot="${current_dir}/snapshots/${veery_commit}_${file%.on}.out"

    if [ ! -f "$ref_snapshot" ]; then
        echo -e "${error_start}reference snapshot '${ref_snapshot}' not found${reset_color}"
        rm "$temp_output"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    # Compare outputs, ignoring the commit hash line
    if diff "$temp_output" "$ref_snapshot" > /dev/null; then
        echo -e "${success_start}matches reference${reset_color}"
        rm "$temp_output"
        git checkout "$original_branch" > /dev/null 2>&1
        return 0
    else
        local diff_file="${current_dir}/snapshots/${veery_commit}_${file%.on}_failed_${oneil_commit}.out"
        mv "$temp_output" "$diff_file"

        echo -e "${error_start}MISMATCH detected${reset_color}"
        echo -e "$error_string differences found between current output and reference snapshot"
        echo -e "$error_string check differences with:"
        echo -e "$error_string     \$ diff $diff_file $ref_snapshot"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi
}

# Track overall success/failure
exit_code=0

# Get snapshots for each file for fixed commits
while IFS=" " read -r commit file; do
    if [ -z "$file" ]; then
        echo -e "${error_string} commit ${commit} has no file"
        exit_code=1
        continue
    fi

    check_snapshot "$file" "$commit" || exit_code=1
done < <(grep -v '^\s*#\|^\s*$' "$current_dir/snapshots.txt")


exit $exit_code 