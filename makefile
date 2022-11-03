.PHONY: default
default: dry

# Dry runs all crates.
.PHONY: dry
dry:
	cd macros && cargo publish --dry-run
	cargo publish --dry-run

# Publishes all crates.
.PHONY: pub
pub:
	cd macros && cargo publish
	cargo publish

