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
echo "📄 Copying generated methods.rs..."
METHODS_FILE=$(find target/release/build/battleship-guests-*/out/methods.rs 2>/dev/null | head -n1)
if [ -n "$METHODS_FILE" ]; then
    cp "$METHODS_FILE" prebuilt/methods.rs
    echo "✅ Copied $METHODS_FILE"
else
    echo "❌ Could not find methods.rs"
    exit 1
fi

echo ""
echo "🔍 Image IDs:"
grep "ID:" prebuilt/methods.rs

echo ""
echo "✅ Done! Now commit: git add prebuilt/"
