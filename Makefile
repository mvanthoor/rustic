# Remove this from strings when parsing the TOML-file.
rm_chars = "\ \"\=\n"
rm_name = "s/name//g"
rm_ver = "s/version//g"
brace = "\{"

# Set engine name and version by parsing the TOML file.
name = $(shell cat Cargo.toml | grep -i "name" | tr -d $(rm_chars) | tr A-Z a-z | sed $(rm_name))
ver = $(shell cat Cargo.toml | grep -i "version" | grep -iv $(brace) | tr -d $(rm_chars) | sed $(rm_ver))
full_name = $(name) $(ver)

all:
	echo $(full_name)