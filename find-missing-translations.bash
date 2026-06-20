#!/usr/bin/env bash

readarray -t ALL_TRANSLATIONS < <(rg -e "get_translation\(.*\)" -o --no-filename --no-line-number --no-heading --color=never | rg -e "\".*\"" -o | cut -d \" -f 2 | sort -u)

readarray -t ALL_LOCALES < <(ls locales/*.ftl)

for TRANSLATION in "${ALL_TRANSLATIONS[@]}"; do
    for LOCALE in "${ALL_LOCALES[@]}"; do
	TRANSLATION_MATCH="$(rg "$TRANSLATION" "$LOCALE")"
	if [[ -z "$TRANSLATION_MATCH" ]]; then
	    echo "Translation missing in $LOCALE: $TRANSLATION"
	fi
    done
done
