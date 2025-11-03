#!/bin/bash
set -e

echo "🔨 Building guest programs from source..."
cargo build --release

echo ""
echo "📁 Creating prebuilt directory..."
mkdir -p prebuilt

echo ""
echo "📦 Copying guest binaries..."
cp target/riscv-guest/battleship-guests/battleship-methods/riscv32im-risc0-zkvm-elf/release/*.bin prebuilt/

echo ""
echo "📄 Generating methods.rs with relative paths..."

# Get the Image IDs from the binaries
INIT_BIN="target/riscv-guest/battleship-guests/battleship-methods/riscv32im-risc0-zkvm-elf/release/init.bin"
ROUND_BIN="target/riscv-guest/battleship-guests/battleship-methods/riscv32im-risc0-zkvm-elf/release/round.bin"

# Extract Image IDs from original methods.rs
METHODS_FILE=$(find target/release/build/battleship-guests-*/out/methods.rs 2>/dev/null | head -n1)
INIT_ID=$(grep "INIT_ID:" "$METHODS_FILE" | sed 's/.*\[\(.*\)\];/[\1];/')
ROUND_ID=$(grep "ROUND_ID:" "$METHODS_FILE" | sed 's/.*\[\(.*\)\];/[\1];/')

# Create methods.rs with relative paths
cat > prebuilt/methods.rs << METHODS_EOF
pub const INIT_ELF: &[u8] = include_bytes!("init.bin");
pub const INIT_ID: [u32; 8] = $INIT_ID
pub const ROUND_ELF: &[u8] = include_bytes!("round.bin");
pub const ROUND_ID: [u32; 8] = $ROUND_ID
METHODS_EOF

echo "✅ Created methods.rs with relative paths"
echo ""
echo "🔍 Image IDs:"
cat prebuilt/methods.rs

echo ""
echo "✅ Done! Now commit: git add prebuilt/"
