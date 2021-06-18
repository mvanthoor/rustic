# =========================================================================== #
# This is Rustic's Makefile. It was written and tested in the Bash shell on
# the following operating systems: Windows (MSYS2), MacOS, Linux, Raspberry
# Pi.
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
# If you need more information regarding the build process, take a look
# into /docs/build.md.
# ========================================================================== #

# Normally, a Make target is used to create a file with the target's name.
# A target that is not expected to create a file with the target's name, is
# called "PHONY" (not real, fake). In this Makefile, we use all the targets
# as subroutines, not as a means to create files. Therefore they are all
# listed as phony.
.PHONY: all clean rm-bin rm-target switch-gnu switch-msvc create-dir native bmi2 popcnt old ancient

# Set minimum required Rust version.
rust_min_version = 1.46.0

# Set base dir for the binaries.
base_dir = ./bin

# ===== The code below should not be changed. ===== #

# Actions needed for handling text parsing
to_lowercase = tr A-Z a-z
rm_chars = tr -d "\ \"\=\n"
rm_name = sed "s/name//g"
rm_ver = sed "s/version//g"
rm_features = grep -iv "features ="
rm_nl = tr -d "\n"
rm_release = tr -d "release: "
grep_name = grep -i "name"
grep_version = grep -i "version"
grep_release = grep -i "release"
grep_machine = grep -i "machine model"

# Set engine name and version by parsing the TOML file.
eng_name = $(shell cat Cargo.toml | $(grep_name) | $(rm_chars) | $(to_lowercase) | $(rm_name))
eng_ver = $(shell cat Cargo.toml | $(grep_version) | $(rm_features) | $(rm_chars) | $(rm_ver))

# Get current operating environment.
uname = $(shell uname -a | $(to_lowercase) | $(rm_nl))
rust_found = $(shell command -v rustc | $(rm_nl))
rust_version = $(shell rustc -Vv | $(grep_release) | $(rm_release) | $(rm_nl))
rust_sort = $(sort $(rust_min_version) $(rust_version))
rust_ok = no

# Check if the minimum Rust version is satisfied
ifeq ($(word 1, $(rust_sort)),$(rust_min_version))
	rust_ok = yes
endif

# Find out the OS we are running on
os =
ext =
bits =
strip =

# Windows MSYS2 64-bit
ifeq ($(findstring mingw64,$(uname)),mingw64)
	os = windows
	bits = 64-bit
	ext = .exe
	strip = strip -s
endif

# Windows MSYS2 32-bit
ifeq ($(findstring mingw32,$(uname)),mingw32)
	os = windows
	bits = 32-bit
	ext = .exe
	strip = strip -s
endif

# MacOS Intel 64-bit
ifeq ($(findstring darwin,$(uname)),darwin)
	os = macos
	bits = 64-bit
	ext =
	strip = strip
endif

# Linux
ifeq ($(findstring linux,$(uname)),linux)
# Determine if Linux on Raspberry
model = $(shell dmesg | $(to_lowercase) | $(rm_nl) | $(grep_machine))

# Linux on Raspberry
ifeq ($(findstring raspberry,$(model)),raspberry)
	os = raspberry
	bits = 32-bit
	ext =
	strip = strip -s
endif

# Linux on Intel/AMD
ifneq ($(findstring raspberry,$(model)),raspberry)
	os = linux
	bits = 64-bit
	ext =
	strip = strip -s
endif
endif

# Create the output directory if it doesn't exist
dir_exists = $(shell ls -l $(base_dir)/ | grep -i $(os) | tr -d "\n")
out_dir = $(base_dir)/$(os)

# Set filenames
out_file = $(eng_name)-$(eng_ver)-$(os)-$(bits)
rel_file = ./target/release/$(eng_name)

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

cpu_level =
compiler_target =
cargo_command =

# Windows 64-bit
ifeq ($(findstring windows,$(os)),windows)
ifeq ($(findstring 64-bit,$(bits)),64-bit)
all: native bmi2 popcnt old ancient
endif
endif

# Linux 64-bit
ifeq ($(findstring linux,$(os)),linux)
ifeq ($(findstring 64-bit,$(bits)),64-bit)
all: native bmi2 popcnt old ancient
endif
endif

# MacOS 64-bit (Intel)
ifeq ($(findstring macos,$(os)),macos)
ifeq ($(findstring 64-bit,$(bits)),64-bit)
all: native bmi2 popcnt old ancient
endif
endif

# Compile one version for Raspberry
ifeq ($(findstring raspberry,$(os)),raspberry)
ifeq ($(findstring 32-bit,$(bits)),32-bit)
all: arm32bit
endif
endif

# Compile one version for Windows 32-bit
ifeq ($(findstring windows,$(os)),windows)
ifeq ($(findstring 32-bit,$(bits)),32-bit)
all: i686
endif
endif

switch-gnu: clean
	rustup default stable-x86_64-pc-windows-gnu

switch-msvc: clean
	rustup default stable-x86_64-pc-windows-msvc

clean: rm-bin rm-target
	
# ===== The targets below are dependencies ===== #

create-dir:
ifeq ($(dir_exists),)
	$(info Creating directory: $(out_dir))
	$(shell mkdir -p $(out_dir))
endif

rm-bin:
	rm -rf ./bin

rm-target:
	rm -rf ./target

native: export RUSTFLAGS = -C target-cpu=native
native: create-dir rm-target
	$(eval cpu_level = native)
	$(eval cargo_command = cargo build --release)
	$(call compile)

bmi2: export RUSTFLAGS = -C target-cpu=haswell
bmi2: create-dir rm-target
	$(eval cpu_level = bmi2)
	$(eval cargo_command = cargo build --release)
	$(call compile)

popcnt: export RUSTFLAGS = -C target-cpu=nehalem
popcnt: create-dir rm-target
	$(eval cpu_level = popcnt)
	$(eval cargo_command = cargo build --release)
	$(call compile)

old: export RUSTFLAGS = -C target-cpu=core2
old: create-dir rm-target
	$(eval cpu_level = old)
	$(eval cargo_command = cargo build --release)
	$(call compile)

ancient: export RUSTFLAGS = -C target-cpu=athlon64
ancient: create-dir rm-target
	$(eval cpu_level = ancient)
	$(eval cargo_command = cargo build --release)
	$(call compile)

arm32bit: create-dir rm-target
	$(eval cpu_level = arm)
	$(eval cargo_command = cargo build --release)
	$(call compile)

i686: export RUSTFLAGS = -C target-cpu=i686
i686: create-dir rm-target
	$(eval cpu_level = i686)
	$(eval compiler_target = i686-pc-windows-gnu)
	$(eval cargo_command = cargo build --release --target=$(compiler_target))
	$(eval rel_file = ./target/$(compiler_target)/release/$(eng_name))
	$(call compile)

# ===== Custom functions ===== #

define compile
$(info Creating: $(out_file)-$(cpu_level)$(ext))
$(cargo_command)
$(strip) $(rel_file)$(ext)
mv $(rel_file)$(ext) $(out_dir)/$(out_file)-$(cpu_level)$(ext)
endef