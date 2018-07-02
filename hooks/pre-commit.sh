#!/bin/sh
ret=0

err() {
    printf '\033[0;31mx %s\033[0m\n' "$*"
    ret=1
}

ok() {
    printf '\033[0;32mv %s\033[0m\n' "$*"
}

which cargo-fmt > /dev/null || {
    err "No cargo-fmt installed"
}

if cargo fmt; then
    ok "rustfmt'd"
else
    err "Can't cargo fmt source"
fi



if out=$(cargo test --color always 2>&1); then
    ok "tests ok"
else
    err "tests failed"
    echo "$out"
    echo
fi

untracked=$(git ls-files --others --exclude-standard)

if [ -z "$untracked" ]; then
    ok "no untracked files"
else
    err "you have some untracked files"
    echo "$untracked"
fi

exit $ret
