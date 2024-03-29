echo "Generating packages..."
# Perform in parallel
# parallel -a packages.txt --jobs 20 ./gen_one.sh
mcvm_tools gen-pkg-batched config.json $@
