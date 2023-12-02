cargo install bootimage
rustup override set nightly
rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu
rustup component add llvm-tools-preview
cargo build --manifest-path=/workspaces/RL4/user_space/ping_pong/Cargo.toml