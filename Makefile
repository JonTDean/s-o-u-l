.PHONY: check governance format-check lint test preview lenia verify-lenia automata verify-automata

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

lenia:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WGPU_BACKEND=vulkan cargo run -- --lenia-demo

verify-lenia:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WGPU_BACKEND=vulkan SOUL_LENIA_VERIFY="$${SOUL_LENIA_VERIFY:-/tmp/soul-lenia-verification}" cargo run -- --lenia-demo

automata:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WGPU_BACKEND=vulkan cargo run -- --automata-demo

verify-automata:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WGPU_BACKEND=vulkan SOUL_AUTOMATA_VERIFY="$${SOUL_AUTOMATA_VERIFY:-/tmp/soul-automata-verification}" cargo run -- --automata-demo

# Fresh output directories prevent stale evidence from qualifying a rerun.
experiment:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WGPU_BACKEND=vulkan cargo run -- --experiment-demo

verify-experiment:
	env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET WGPU_BACKEND=vulkan SOUL_EXPERIMENT_VERIFY="$${SOUL_EXPERIMENT_VERIFY:-/tmp/soul-experiment-verification}" cargo run -- --experiment-demo

benchmark-sparse:
	RAYON_NUM_THREADS="$${RAYON_NUM_THREADS:-4}" cargo test -p simulation_kernel --release sparse_benchmark -- --ignored --nocapture
