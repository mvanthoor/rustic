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

# Determine if everything is correct
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