echo "old solver"
cargo +nightly check
cargo clean -p hybrid-array
echo "new solver"
RUSTFLAGS="-Znext-solver=globally" cargo +nightly check
cargo clean -p hybrid-array
