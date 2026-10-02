#!/bin/bash
# Start temp-app with safe data directory and library path
#
# Why TELEPATHY_DATA_DIR:
#   The default location (~/Library/Application Support/com.telepathy.app/)
#   is blocked by the TRAE sandbox, which causes SQLite readonly errors.
#   Using a project-local path avoids the sandbox restriction during development.
#
# Why DYLD_LIBRARY_PATH:
#   The app needs to find the local llama.cpp dylibs in target/debug.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default to a project-local data dir; allow override via env var
export TELEPATHY_DATA_DIR="${TELEPATHY_DATA_DIR:-$PROJECT_ROOT/.data}"
mkdir -p "$TELEPATHY_DATA_DIR"

# Remove any xattr on the existing system db to avoid SQLITE_READONLY_DBMOVED
DB_PATH="$HOME/Library/Application Support/com.telepathy.app/telepathy.db"
xattr -d com.apple.provenance "$DB_PATH" 2>/dev/null || true

cd "$PROJECT_ROOT" || exit 1
exec env DYLD_LIBRARY_PATH="$PROJECT_ROOT/target/debug" "$PROJECT_ROOT/target/debug/temp-app" "$@"
