#!/bin/bash
set -e
if ! [ "." -ef "$(git rev-parse --show-toplevel)" ]; then
    echo "Not in git root"
    exit 1
fi

for hook in hooks/*; do
    if ! [ "$hook" -ef "$0" ]; then
        file=${hook##*/}
        file=${file%.*}
        ln -sfv -- "../../$hook" ".git/hooks/$file"
    fi
done
