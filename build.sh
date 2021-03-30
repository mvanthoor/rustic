#!/bin/bash

# Set engine name and version.
ENGINE=$(cat Cargo.toml | grep -i "name" | tr -d "name= \"\n" | tr A-Z a-z)
VERSION=$(cat Cargo.toml | grep -i "version" | grep -iv "\{" | tr -d "version= \"\n")
FULL_NAME="$ENGINE alpha $VERSION"

# Set minimum required Rust version.
RUST_MIN_VERSION="1.46.0"

# Set base dir for the binaries.
BASE_DIR="./bin"

# ========== NEXT PART SHOULD NOT BE CHANGED ========== #

# Get current operating environment.
UNAME=$(uname -a | tr A-Z a-z | tr -d "\n")
RUST_FOUND=$(command -v rustc | tr -d "\n")

# These variables will be set below.
OS=""
BIT="64-bit"
RUST_VERSION=""
DIR=""
RESULT=""
FILENAME=""
EXE=""
TYPE=""
FLAGS=""
TARGET=""
ERROR=""

# Check for which operating system we should build.
if [ "$ERROR" == "" ]; then
  # Determine which operating system we should build for.
  case $UNAME in
    "mingw64"*)
      OS="windows"
      EXE=".exe"
      ;;
    "mingw32"*)
      OS="windows"
      EXE=".exe"
      BIT="32-bit"
      ;;
    "darwin"*)
      OS="macos"
      ;;
    "linux"*)
      OS="linux"
      ;;
  esac

  # We're on an OS we can't automatically build for
  if [ "$OS" == "" ]; then
    ERROR="Cannot determine operating system."
  fi
fi

# We determined the operating system. See if Rust is available.
if [ "$ERROR" == "" ]; then
  if [ "$RUST_FOUND" == "" ]; then
    ERROR="Rust is not found."
  fi
fi

# Rust is available. Check the version.
if [ "$ERROR" == "" ]; then
  RUST_VERSION=$(rustc -Vv | grep -i "release" | tr -d "release: " | tr -d "\n")
  # Minimal version must be lower or equal than installed version
  if ! [ "$RUST_MIN_VERSION" \< "$RUST_VERSION" -o  "$RUST_MIN_VERSION" == "$RUST_VERSION" ]; then
    ERROR="Rust version is $RUST_VERSION. Must be at least $RUST_MIN_VERSION."
  fi
fi

# No errors for OS and Rust version. We create the
# directory that will hold the binaries.
if [ "$ERROR" == "" ]; then
  # Directory name to create.
  DIR="$BASE_DIR/$OS"

  # Directory either exists or we create it.
  if [ -d "$DIR" ]; then
    echo "Existing bin-directory: $DIR"
  else
    echo "Creating bin-directory: $DIR"
    mkdir -p "$DIR"
  fi

  # If it still doesn't exist, we failed to create it.
  if [ ! -d "$DIR" ]; then
    ERROR="Creating directory $DIR failed."
  fi
fi

# Determine which type of executable to build.
if [ "$ERROR" == "" ]; then
  T=$(echo "$1" | tr A-Z a-z | tr -d "\n")

  if [ "$BIT" == "32-bit" ]; then
    echo "Only compiling 'generic' cpu-type for 32-bit."
    TYPE="generic"
  fi

  if [ "$BIT" == "64-bit" ]; then
    case "$T" in
      "generic")
        TYPE="generic"
        ;;
      "old")
        TYPE="old"
        ;;
      "popcnt")
        TYPE="popcnt"
        ;;
      "bmi2")
        TYPE="bmi2"
        ;;
      "native")
        TYPE="bmi2"
        ;;
    esac
  fi
fi

# No errors: print information
if [ "$ERROR" == "" ]; then
  echo "Build for: $OS $BIT"
  if [ "$TYPE" != "" ]; then
    echo "Compile for CPU-type: $TYPE"
  else
    TYPE="all"
    echo "Compile for all CPU-types"
  fi
  echo "Rust version: $RUST_VERSION"
  echo "Building: $FULL_NAME"
else
  echo "Error: $ERROR"
fi

# Function used below, after setting the build flags.
function create_build {
    rm -rf ./target
    export RUSTFLAGS="$FLAGS"

    if [ "$TARGET" == "" ]; then
      cargo build --release
    else
      cargo build --release "$TARGET"
    fi

    strip -s "$RESULT"
    mv -f "$RESULT" "$FILENAME"
}


# Start building 32-bit
if [ "$ERROR" == "" ] && [ "$BIT" == "64-bit" ]; then
  RESULT="./target/release/rustic.exe"
  FULL_NAME=$(echo "$FULL_NAME" | tr " " "-")

  if [ "$TYPE" == "all" ] || [ "$TYPE" == "generic" ]; then
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-generic$EXE"
    FLAGS="-C target-cpu=athlon64"
    create_build
  fi

  if [ "$TYPE" == "all" ] || [ "$TYPE" == "old" ]; then
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-old$EXE"
    FLAGS="-C target-cpu=core2"
    create_build
  fi

  if [ "$TYPE" == "all" ] || [ "$TYPE" == "popcnt" ]; then
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-popcnt$EXE"
    FLAGS="-C target-cpu=nehalem"
    create_build
  fi

  if [ "$TYPE" == "all" ] || [ "$TYPE" == "bmi2" ]; then
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-bmi2$EXE"
    FLAGS="-C target-cpu=haswell"
    create_build
  fi

  if [ "$TYPE" == "all" ] || [ "$TYPE" == "native" ]; then
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-native$EXE"
    FLAGS="-C target-cpu=native"
    create_build
  fi
fi

# Start building 32-bit
if [ "$ERROR" == "" ] && [ "$BIT" == "32-bit" ]; then
  if [ "$TYPE" == "all" ] || [ "$TYPE" == "generic" ]; then
    RESULT="./target/i686-pc-windows-gnu/release/rustic.exe"
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-generic$EXE"
    FLAGS="-C target-cpu=i686"
    TARGET="--target=i686-pc-windows-gnu"
    create_build
  fi
fi