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

# script colors
error_start="\e[31m"
info_start="\e[34m"
success_start="\e[32m"
reset_color="\e[0m"
error_string="[${error_start}ERROR${reset_color}]"
info_string="[${info_start}INFO ${reset_color}]"

current_dir=$(pwd)
model_dir="$1"

if [ -z "$model_dir" ]; then
    echo -e "$error_string model directory not provided"
    echo -e "Usage: $0 <model_dir>"
    exit 1
fi

# Ensure snapshots directory exists
mkdir -p "${current_dir}/snapshots"

cd "$model_dir"

# Get a snapshot of the current output of the interpreter for a given file on a given commit
get_snapshot() {
    local file="$1"
    local veery_commit="$2"

    local original_branch=$(git branch --show-current)

    echo -ne "$info_string getting snapshot for '${file}' on commit '${veery_commit}' ... "

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

    # Set up output file path
    local output_file="${current_dir}/snapshots/${veery_commit}_${file%.on}.out"

    # Create or clear the output file
    > "$output_file"

    # Generate header
    echo "Run on commit ${veery_commit} of file ${file}" >> "$output_file"
    echo "--------------------------------" >> "$output_file"

    # Run oneil and capture output
    if yes "quit" | oneil regression-test "$file" >> "$output_file" 2>&1; then
        echo -e "${success_start}done${reset_color}"
        echo -e "$info_string output saved to: ${output_file}"
    else
        echo -e "${error_start}failed to run oneil on ${file} on commit ${veery_commit}${reset_color}"
        rm "$output_file"
        git checkout "$original_branch" > /dev/null 2>&1
        return 1
    fi

    git checkout "$original_branch" > /dev/null 2>&1
    return 0
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

    get_snapshot "$file" "$commit" || exit_code=1
done < <(grep -v '^\s*#\|^\s*$' "$current_dir/snapshots.txt")

exit $exit_code