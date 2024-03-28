echo "Generating packages..."
# Perform in parallel
# We have to limit the number of jobs so that Modrinth doesn't complain about too many requests
parallel -a packages.txt --jobs 20 ./gen_one.sh
