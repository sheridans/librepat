#!/bin/sh
set -eu

find . -path ./target -prune -o -name '*.rs' -type f -print | (
    status=0
    while IFS= read -r file; do
        lines=$(wc -l < "$file")
        if [ "$lines" -gt 300 ]; then
            echo "$file has $lines lines; maximum is 300" >&2
            status=1
        fi
    done
    exit "$status"
)
