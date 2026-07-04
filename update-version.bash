#!/usr/bin/env bash

if [[ -z "$1" ]]; then
    echo "Need a version in the first positional argument";
    exit 255
fi

LATEST_TAG="$(git describe --tags --abbrev=0)"
LATEST_TAG="$(echo "$LATEST_TAG" | sed -E 's/\./\\./g')"

readarray -t MATCHED_FILES < <(rg -l "$LATEST_TAG" --glob "!Cargo.lock" --color=never)

for FILE in "${MATCHED_FILES[@]}"; do
    sed -E -i "s/$LATEST_TAG/$1/g" "$FILE"
done
