#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OS_DIR="$SCRIPT_DIR/cpd_os"

usage() {
  echo "Usage: $0 {-b|-c|-cu|-h}"
  echo ""
  echo "  -b     Build for x86_64 target"
  echo "  -c     Clean project"
  echo "  -cu    Update dependencies"
  echo "  -h     Show this help message"
}

enter_dir() {
  pushd "$1" >/dev/null || {
    echo "Cannot enter $1"
    exit 1
  }
}

leave_dir() {
  popd >/dev/null
}

build() {
  enter_dir "$OS_DIR"
  cargo build --target x86_64-unknown-none || {
    echo "cargo build failed"
    leave_dir
    exit 1
  }
  leave_dir
}

update() {
  enter_dir "$OS_DIR"
  cargo update || {
    echo "Update failed"
    leave_dir
    exit 1
  }
  leave_dir
}

clean() {
  echo "Cleaning os..."
  enter_dir "$OS_DIR"
  cargo clean || {
    echo "OS clean failed"
    leave_dir
    exit 1
  }
  leave_dir
}

case "$1" in
-b) build ;;
-c) clean ;;
-cu) update ;;
-h) usage ;;
*)
  usage
  exit 1
  ;;
esac
