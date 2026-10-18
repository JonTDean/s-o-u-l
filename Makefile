.PHONY: check governance format-check lint test preview

check: governance format-check lint test

governance:
	python3 -m unittest discover -s tools/structure/tests -t tools/structure
	python3 tools/structure/audit.py

format-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
	RUSTDOCFLAGS="$${RUSTDOCFLAGS:-} -C rpath=yes" cargo test --workspace --all-features

preview:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET cargo run -- --voxel-demo
