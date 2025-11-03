use std::path::Path;
use std::env;
use std::fs;

fn main() {
    let prebuilt_methods = Path::new("../prebuilt/methods.rs");
    let use_prebuilt = prebuilt_methods.exists() 
        && env::var("FORCE_REBUILD_GUESTS").is_err();

    if use_prebuilt {
        println!("cargo:warning=✅ Using pre-built guest binaries");
        println!("cargo:warning=   All players will have IDENTICAL Image IDs");
        
        // Copy methods.rs to OUT_DIR
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest = Path::new(&out_dir).join("methods.rs");
        fs::copy(prebuilt_methods, &dest)
            .expect("Failed to copy prebuilt methods.rs");
        
        // Copy binaries to expected location
        let bin_dir = Path::new("../target/riscv-guest/battleship-guests/battleship-methods/riscv32im-risc0-zkvm-elf/release");
        fs::create_dir_all(bin_dir).ok();
        
        for bin in &["init.bin", "round.bin"] {
            let src = Path::new("../prebuilt").join(bin);
            let dst = bin_dir.join(bin);
            if src.exists() {
                fs::copy(&src, &dst).ok();
            }
        }
        
        println!("cargo:rerun-if-changed=../prebuilt/methods.rs");
        println!("cargo:rerun-if-changed=../prebuilt/init.bin");
        println!("cargo:rerun-if-changed=../prebuilt/round.bin");
    } else {
        println!("cargo:warning=🔨 Building from source");
        risc0_build::embed_methods();
    }
}