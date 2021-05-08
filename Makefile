# =========================================================================== #
# This is Rustic's Makefile. It was written and tested in the Bash shell on
# the following operating systems: Windows (MSYS2), MacOS, Linux.
#
# The Makefile checks all the necessary conditions and sets all the
# parameters for building the engine. Except for the minimum required Rust
# version, the build script gets all the needed information from the
# envorinment, or the Cargo.toml file.
#
# The Makefile only starts building Rustic if all requirements have been
# fulfilled. As soon as an unmet requirement is found, this will be
# reported and the engine will not be built.
#
# Further comments describing each step can be found below.
#
# If Rust is installed correctly but the Makefile does not work for
# whatever reason, it should still be possible to compile the engine
# manually using the following commands:
#
# cargo build --release
#
# strip -s ./target/release/rustic-alpha
#
# (On Windows, the name of the executable will be rustic-alpha.exe.) The
# "strip" command is optional; it will strip debug symbols from the binary
# to make it smaller. If you also wish to build the engine for your
# specific CPU, then set the RUSTFLAGS="-C target-cpu=core2" environment
# variable before running the cargo command.
# ========================================================================== #

# Set minimum required Rust version.
rust_min_version = 1.46.0

# Set base dir for the binaries.
base_dir = ./bin

# ===== The code below should not be changed. ===== #

# Remove this from strings when parsing the TOML-file.
rm_chars = "\ \"\=\n"
rm_name = "s/name//g"
rm_ver = "s/version//g"
rm_brace = "\{"

# Set engine name and version by parsing the TOML file.
eng_name = $(shell cat Cargo.toml | grep -i "name" | tr -d $(rm_chars) | tr A-Z a-z | sed $(rm_name))
eng_ver = $(shell cat Cargo.toml | grep -i "version" | grep -iv $(rm_brace) | tr -d $(rm_chars) | sed $(rm_ver))

# Get current operating environment.
uname = $(shell uname -a | tr A-Z a-z | tr -d "\n")
rust_found = $(shell command -v rustc | tr -d "\n")
rust_version = $(shell rustc -Vv | grep -i "release" | tr -d "release: " | tr -d "\n")
rust_sort = $(sort $(rust_min_version) $(rust_version))
rust_ok = no

# Check if the minimum Rust version is satisfied
ifeq ($(word 1, $(rust_sort)),$(rust_min_version))
	rust_ok = yes
endif

# Find out the OS we are running on
os =
exe =
bits =

ifeq ($(findstring mingw64,$(uname)),mingw64)
	os = windows
	bits = 64-bit
	exe = .exe
endif
ifeq ($(findstring mingw32,$(uname)),mingw32)
	os = windows
	bits = 32-bit
	exe = .exe
endif
ifeq ($(findstring darwin,$(uname)),darwin)
	os = macos
	bits = 64-bit
	exe =
endif
ifeq ($(findstring linux,$(uname)),linux)
	os = linux
	bits = 64-bit
	exe =
endif

# Set output directory and file name
output_dir = $(base_dir)/$(os)
file_name = $(eng_name)-$(eng_ver)-$(os)-$(bits)

# Determine if everything is correct. If not, abort.
ifeq ($(os),)
$(error Unknown OS: This operating system is not supported)
endif
ifeq ($(bits),)
$(error Unknown architecture: the system must be 32-bit or 64-bit)
endif
ifeq ($(rust_found),)
$(error Rust not found or not installed)
endif
ifeq ($(rust_ok),no)
$(error Rust version not supported)
endif

# ===== Main targets ===== #

all:
	$(info Compiling...)

gnu: clean switch-gnu all

msvc: clean switch-msvc all

clean:
	rm -rf ./bin
	rm -rf ./target

# ===== The targets below are dependencies ===== #

switch-gnu:
	rustup default stable-x86_64-pc-windows-gnu

switch-msvc:
	rustup default stable-x86_64-pc-windows-msvc