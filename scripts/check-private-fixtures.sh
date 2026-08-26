#!/bin/sh
set -eu

tracked=$(git ls-files '*.FLK' '*.flk' '*.PAT' '*.pat' '*.pdf' '*.librepat')
if [ -n "$tracked" ]; then
    echo "private fixture-like files are tracked:" >&2
    echo "$tracked" >&2
    exit 1
fi

if rg -i -l --glob '!PLAN.md' --glob '!CONTRIBUTING.md' \
    --glob '!scripts/check-private-fixtures.sh' \
    'DMS\.FLK|SLL\.FLK|PAT Report Passes And Fails|certificate\.pdf|patreport\.pdf|fcc\.PAT' .; then
    echo "private reference filename found in tracked project content" >&2
    exit 1
fi

