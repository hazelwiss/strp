.PHONY: default
default: dry

# Dry runs all crates.
.PHONY: dry
dry:
	cargo publish --dry-run

# Publishes all crates.
.PHONY: pub
pub:
	cargo publish

