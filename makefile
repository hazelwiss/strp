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
	cargo readme > README.md
	cd macros && cargo publish
	cargo publish
	git push

