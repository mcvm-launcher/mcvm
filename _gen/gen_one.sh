# Read properties from the line
LINE=($1)
PKG_NAME=${LINE[0]}
PKG_SOURCE=${LINE[1]}
PKG_ID=${LINE[2]}

# Print info
echo " - Generating package '$PKG_NAME'..."
# echo "      From '$PKG_SOURCE' with ID '$PKG_ID'..."

# Create the generation config if it is not created already
PKG_GEN_CONFIG="./configs/$PKG_NAME.json"
if ! test -f $PKG_GEN_CONFIG; then
	echo "{}" > $PKG_GEN_CONFIG;
fi

# Generate the package
OUTPUT=$(mcvm_tools gen-pkg $PKG_SOURCE $PKG_ID -c $PKG_GEN_CONFIG)
# Write the package file with JSON formatting
echo $OUTPUT | jq --tab > "../std/api/mcvm/pkg/$PKG_NAME.json"
