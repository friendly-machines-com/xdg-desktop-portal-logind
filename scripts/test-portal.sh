#!/bin/sh
# Real frontend + real logind, isolated SESSION bus and temporary configuration.
set -eu

project=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project"

if ! command -v dbus-run-session >/dev/null 2>&1; then
    printf '%s\n' 'Missing dbus-run-session. On Guix, run:' \
        'guix shell dbus xdg-desktop-portal -- sh scripts/test-portal.sh idle 45' >&2
    exit 1
fi

frontend=${PORTAL_FRONTEND:-}
if [ -z "$frontend" ]; then
    frontend=$(command -v xdg-desktop-portal || true)
fi
if [ -z "$frontend" ]; then
    for candidate in \
        "${GUIX_ENVIRONMENT:-/nonexistent}/libexec/xdg-desktop-portal" \
        /usr/libexec/xdg-desktop-portal \
        /usr/lib/xdg-desktop-portal
    do
        if [ -x "$candidate" ]; then frontend=$candidate; break; fi
    done
fi
if [ -z "$frontend" ] || [ ! -x "$frontend" ]; then
    printf '%s\n' 'Cannot locate xdg-desktop-portal. Set PORTAL_FRONTEND to its absolute path.' >&2
    exit 1
fi

client_example=portal
if [ "${1:-}" = --client-death ]; then
    client_example=client_death
    shift
fi
cargo build --locked --bin xdg-desktop-portal-logind --example portal --example client_death
cargo build --locked --offline --package xtask
# Honor a custom CARGO_TARGET_DIR; make paths absolute for D-Bus activation.
target=${CARGO_TARGET_DIR:-target}
case "$target" in /*) ;; *) target="$project/$target" ;; esac

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' 0
trap 'exit 130' INT
trap 'exit 143' TERM
prefix="$tmp/prefix"
"$target/debug/xtask" install --binary "$target/debug/xdg-desktop-portal-logind" --prefix "$prefix"
portals="$prefix/share/xdg-desktop-portal/portals"
mkdir -p "$tmp/config/xdg-desktop-portal" "$tmp/cache" "$tmp/runtime"
chmod 700 "$tmp/runtime"
printf '%s\n' '[preferred]' 'default=logind' \
    'org.freedesktop.impl.portal.Inhibit=logind' > "$portals/portals.conf"
# Older portal versions find configuration via XDG_CONFIG_HOME instead.
cp "$portals/portals.conf" "$tmp/config/xdg-desktop-portal/portals.conf"

printf '%s\n' 'Testing on a private session bus. Your desktop portal configuration is unchanged.' \
    'This test DOES acquire a real system logind/elogind inhibitor.'

PORTAL_FRONTEND="$frontend" TEST_TMP="$tmp" TEST_CLIENT="$target/debug/examples/$client_example" \
XDG_DESKTOP_PORTAL_DIR="$portals" XDG_CONFIG_HOME="$tmp/config" \
XDG_DATA_HOME="$prefix/share" XDG_CACHE_HOME="$tmp/cache" XDG_RUNTIME_DIR="$tmp/runtime" \
XDG_CURRENT_DESKTOP=LogindTest \
dbus-run-session -- sh -c '
    set -eu
    "$PORTAL_FRONTEND" --verbose > "$TEST_TMP/frontend.log" 2>&1 &
    frontend_pid=$!
    trap '\''kill "$frontend_pid" 2>/dev/null || true; wait "$frontend_pid" 2>/dev/null || true'\'' 0
    trap '\''exit 130'\'' INT
    trap '\''exit 143'\'' TERM
    status=0
    "$TEST_CLIENT" "$@" || status=$?
    if [ "$status" -ne 0 ]; then
        printf "\nPortal frontend log:\n" >&2
        cat "$TEST_TMP/frontend.log" >&2
    fi
    exit "$status"
' sh "$@"
