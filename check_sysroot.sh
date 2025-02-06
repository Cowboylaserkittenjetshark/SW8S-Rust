#!/usr/bin/env bash
set -e

# Get absolute path with no trailing slash
# Important for how links are fixed
destdir="$(realpath "$1")"
destdir="${destdir%"/"}"
if [ ! -d "$destdir" ]; then
    echo "Destination folder does not exist."
    exit 1
fi

echo "Checking for broken links"
while read l; do
    if [ ! -L "$l" ] || [ ! -e "$l" ]; then
        t=$(readlink "$l")
        echo "Found broken link in sysroot: $l -> $t"
    fi
done <<< "$(find "$destdir" -type l)"
