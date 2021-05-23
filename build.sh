#!/bin/bash

# =========================================================================== #
# This is Rustic's build script. It was written and tested in the Bash
# shell on the following operating systems: Windows (MSYS2), MacOS, Linux.
# Care was taken to stay POSIX-compliant, but running this script in a
# different shell or on a different operating system than the ones
# mentioned above, is unsupported.
#
# The script checks all the necessary conditions and sets all the
# parameters for building the engine. Except for the minimum required Rust
# version, the build script gets all the needed information from the
# envorinment, or the Cargo.toml file.
#
# The script only starts building Rustic if all requirements have been
# fulfilled. As soon as an unmet requirement is found, the script will
# report this and terminate.
#
# Further comments describing each step can be found below.
#
# If Rust is installed correctly but the build script does not work for
# whatever reason, it should still be possible to compile the engine
# manually using the following commands:
#
# cargo build --release
#
# strip -s ./target/release/rustic-alpha
#
# The "strip" command is optional; it will strip debug symbols from the
# binary to make it smaller. Add ".exe", without quotes, at the end of this
# command if on Windows. (In Windows, this command is only available in a
# seperately installed BASH shell such as MSYS2.)
# ========================================================================== #

# Remove this from strings when parsing the TOML-file.
RM_CHARS="\ \"\=\n"
RM_NAME="s/name//g"
RM_VER="s/version//g"

# Set engine name and version.
NAME=$(cat Cargo.toml | grep -i "name" | tr -d "$RM_CHARS" | tr A-Z a-z | sed "$RM_NAME")
VERSION=$(cat Cargo.toml | grep -i "version" | grep -iv "\{" | tr -d "$RM_CHARS" | sed "$RM_VER")
FULL_NAME="$NAME $VERSION"

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

  # If we are on Linux, see if it's Raspbian
  if [ "$OS" == "linux" ]; then
    RM_NAME="s/pretty_name=//g"
    WHICH_LINUX=$(cat /etc/\*-release | tr A-Z a-z | grep -i "pretty" | sed "$RM_NAME" | tr -d \")
    case $WHICH_LINUX in
      "ras"*)
        OS="raspberry"
        ;;
    esac
  fi

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
        TYPE="native"
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


if [ "$ERROR" == "" ] && [ "$BIT" == "64-bit" ]; then
  RESULT="./target/release/$NAME$EXE"
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

# Start building Windows 32-bit version
if [ "$ERROR" == "" ] && [ $OS == "windows" ] && [ "$BIT" == "32-bit" ]; then
  if [ "$TYPE" == "all" ] || [ "$TYPE" == "generic" ]; then
    RESULT="./target/i686-pc-windows-gnu/release/rustic.exe"
    FILENAME="$DIR/$FULL_NAME-$OS-$BIT-generic$EXE"
    FLAGS="-C target-cpu=i686"
    TARGET="--target=i686-pc-windows-gnu"
    create_build
  fi
fi