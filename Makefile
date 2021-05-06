# Set minimum required Rust version.
rust_min_version = "1.46.0"

# Set base dir for the binaries.
base_dir = "./bin"

# ===== The code below should not be changed. ===== #

# Remove this from strings when parsing the TOML-file.
rm_chars = "\ \"\=\n"
rm_name = "s/name//g"
rm_ver = "s/version//g"
rm_brace = "\{"

# Set engine name and version by parsing the TOML file.
name = $(shell cat Cargo.toml | grep -i "name" | tr -d $(rm_chars) | tr A-Z a-z | sed $(rm_name))
ver = $(shell cat Cargo.toml | grep -i "version" | grep -iv $(rm_brace) | tr -d $(rm_chars) | sed $(rm_ver))
full_name = $(name) $(ver)

# Get current operating environment.
uname = $(shell uname -a | tr A-Z a-z | tr -d "\n")
rust_found = $(shell command -v rustc | tr -d "\n")

all:
	echo $(full_name)
	echo $(uname)
	echo $(rust_found)