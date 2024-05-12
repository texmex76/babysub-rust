#!/bin/bash

# Change to the directory with the files
cd sc2016

# File to store the results
output_file="../bench.txt"
echo "" > "$output_file"  # Clear the output file

# Loop over all files in the current directory
for file in *; do
    echo "Processing $file..."

    # Run babysub with a timeout
    start_time=$(date +%s.%N)
    timeout 300 babysub "$file" > /dev/null 2>&1
    exit_status=$?
    if [ $exit_status -eq 124 ]; then
        babysub_time="-"
    elif [ $exit_status -ne 0 ]; then
        babysub_time="err"
    else
        end_time=$(date +%s.%N)
        babysub_time=$(echo "$end_time - $start_time" | bc)
    fi

    # Run babysub-rust with a timeout
    start_time=$(date +%s.%N)
    timeout 300 babysub-rust "$file" > /dev/null 2>&1
    exit_status=$?
    if [ $exit_status -eq 124 ]; then
        babysub_rust_time="-"
    elif [ $exit_status -ne 0 ]; then
        babysub_rust_time="err"
    else
        end_time=$(date +%s.%N)
        babysub_rust_time=$(echo "$end_time - $start_time" | bc)
    fi

    # Write results to the output file
    echo "$file $babysub_time $babysub_rust_time" >> "$output_file"
done

echo "Benchmarking complete."
