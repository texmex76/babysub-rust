#!/bin/bash

# Change to the directory with the files
cd sc2016

# File to store the results
output_file="../bench.txt"
echo "" > "$output_file"  # Clear the output file

# Loop over all files in the current directory
for file in *; do
    echo "Processing $file..."

    # Run babysub with a timeout and capture output
    start_time=$(date +%s.%N)
    babysub_output=$(timeout 300 babysub "$file" 2>&1)
    exit_status=$?
    if [ $exit_status -eq 124 ]; then
        babysub_time="-"
        babysub_hash="timeout"
    elif [ $exit_status -ne 0 ]; then
        babysub_time="err"
        babysub_hash="error"
    else
        end_time=$(date +%s.%N)
        babysub_time=$(echo "$end_time - $start_time" | bc)
        babysub_hash=$(echo "$babysub_output" | rg "hash-signature" | awk '{print $3}')
    fi

    # Run babysub-rust with a timeout and capture output
    start_time=$(date +%s.%N)
    babysub_rust_output=$(timeout 300 babysub-rust "$file" 2>&1)
    exit_status=$?
    if [ $exit_status -eq 124 ]; then
        babysub_rust_time="-"
        babysub_rust_hash="timeout"
    elif [ $exit_status -ne 0 ]; then
        babysub_rust_time="err"
        babysub_rust_hash="error"
    else
        end_time=$(date +%s.%N)
        babysub_rust_time=$(echo "$end_time - $start_time" | bc)
        babysub_rust_hash=$(echo "$babysub_rust_output" | rg "hash-signature" | awk '{print $3}')
    fi

    # Compare hashes and determine the output
    if [[ "$babysub_hash" == "$babysub_rust_hash" ]]; then
        hash_comparison="same"
    else
        hash_comparison="different"
    fi

    # Write results to the output file
    echo "$file $babysub_time $babysub_rust_time $hash_comparison" >> "$output_file"
done

echo "Benchmarking complete."
