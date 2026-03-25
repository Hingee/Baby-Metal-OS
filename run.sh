#!/bin/bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OS_DIR="$SCRIPT_DIR/cpd_os"

usage() {
  echo "Usage: $0 {-rq|-i|-b|-c|-cu|-h}"
  echo ""
  echo "  -rq [other_args] -- [qemu_args]   Build and run kernel in QEMU"
  echo "  -i  [other_args]                  Build a bootable image"
  echo "  -b                                Build for x86_64 target"
  echo "  -c                                Clean project"
  echo "  -cu                               Update dependencies"
  echo "  -h                                Show this help message"
}

enter_dir() {
  cd "$1" || {
    echo "Cannot enter $1"
    exit 1
  }
}

leave_dir() {
  cd "$SCRIPT_DIR"
}

build() {
  enter_dir "$OS_DIR"
  cargo build || {
    echo "cargo build failed"
    leave_dir
    exit 1
  }
  leave_dir
}

buildImage() {
  local other_args=("$@")
  enter_dir "$OS_DIR"
  cargo bootimage "${other_args[@]}" || {
    echo "cargo bootimage failed"
    leave_dir
    exit 1
  }
  leave_dir
}

runQEMU() {
  local other_args=()
  local qemu_args=()
  local found_sep=0
  for arg in "$@"; do
    if [[ "$arg" == "--" ]]; then
      found_sep=1
    elif [[ $found_sep -eq 0 ]]; then
      other_args+=("$arg")
    else
      qemu_args+=("$arg")
    fi
  done

  enter_dir "$OS_DIR"
  cargo run "${other_args[@]}" -- "${qemu_args[@]}" || {
    echo "cargo xrun failed"
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

cmd="$1"
shift
case "$cmd" in
-rq) runQEMU "$@" ;;
-i) buildImage "$@" ;;
-b) build ;;
-c) clean ;;
-cu) update ;;
-h) usage ;;
*)
  usage
  exit 1
  ;;
esac
