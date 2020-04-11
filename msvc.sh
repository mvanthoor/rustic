dir="./target"

if [ -d "$dir" ]; then
  echo "Target folder exists. Deleting..."
  rm -rf ${dir}
fi

echo "Switching to MSVC-toolchain..."
rustup default stable-x86_64-pc-windows-msvc

echo "Building Rustic debug build..."
cargo build

echo "Building Rustic release build..."
cargo build --release