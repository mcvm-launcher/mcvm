echo "Generating packages..."
# Perform in parallel
parallel -a packages.txt ./gen_one.sh
