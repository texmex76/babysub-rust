#!/bin/bash

# Check if input file is provided
if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <input_file>"
    exit 1
fi

input_file="$1"

# Prepare data files for plotting
grep -v ' err ' "$input_file" | awk '{print $2}' | sort -n > babysub_times.txt
grep -v ' err ' "$input_file" | awk '{print $3}' | sort -n > babysub_rust_times.txt

# Create a gnuplot script
cat << EOF > plot_script.gp
    # Set the output file and type
    set terminal png size 800,600
    set output 'output.png'

    # Set labels and title
    set xlabel 'Cumulative Time (s)'
    set ylabel 'Number of solved instances'
    set title 'babysub vs babysub-rust'

    # Set the grid
    set grid

    # Set styles
    set style data linespoints
    set key outside

    # Define two colors
    set style line 1 lc rgb 'red' pt 5  # red with circle
    set style line 2 lc rgb 'blue' pt 7 # blue with square

    # Plot the data using inline awk commands to calculate cumulative time
    plot "< awk '{sum += \$1; print sum, NR}' babysub_times.txt" title 'babysub' with linespoints linestyle 1, \\
         "< awk '{sum += \$1; print sum, NR}' babysub_rust_times.txt" title 'babysub-rust' with linespoints linestyle 2
EOF

# Run gnuplot
gnuplot plot_script.gp

# Clean up
rm plot_script.gp
