#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

xtr ../src/*.rs -o "waytrogen.pot"

while read -r lang; do
    msgmerge "$lang.po" "waytrogen.pot" -U 
done < "LINGUAS"
