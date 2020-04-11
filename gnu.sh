dir="./target"

if [ -d "$dir" ]; then
  echo "Target folder exists. Deleting..."
  rm -rf ${dir}
fi

echo "Switching to GNU-toolchain..."
rustup default stable-x86_64-pc-windows-gnu

echo "Building Rustic debug build..."
cargo build

echo "Building Rustic release build..."
cargo build --release
strip -s ./target/release/rustic.exe