# Generates the repository index file

# Grab the base file
BASE_INDEX=$(cat index_base.json)

# Read through files and create the package entries
ENTRIES=""
for FILENAME in ../std/api/mcvm/pkg/*; do
	FILENAME=$(basename $FILENAME)
	CONTENT_TYPE="script"

	# Strip away the filenames
	STRIPPED=$(echo $FILENAME | sed -e 's/\.pkg\.txt//g')
	# If they are the same then the package is declarative
	[ "$STRIPPED" = "$FILENAME" ] && CONTENT_TYPE="declarative"
	STRIPPED=$(echo $FILENAME | sed -e 's/\.json//g')

	# Add the entry to the list
	ENTRIES="$ENTRIES\"$STRIPPED\": {\"path\": \"pkg/$FILENAME\",\"content_type\": \"$CONTENT_TYPE\"},"
done
# Strip the final comma from the entries list
ENTRIES=${ENTRIES::-1}
# echo $ENTRIES

# Modify the index
MODIFIED_INDEX=$(echo $BASE_INDEX | jq --tab ". += {\"packages\": {$ENTRIES}}")
# Write out the index
echo "Index: $MODIFIED_INDEX"
echo "$MODIFIED_INDEX" > ../std/api/mcvm/index.json
