use std::path::Path;
use std::env;
use std::fs;

fn main() {
    let prebuilt_methods = Path::new("../prebuilt/methods.rs");
    let use_prebuilt = prebuilt_methods.exists() 
        && env::var("FORCE_REBUILD_GUESTS").is_err();

    if use_prebuilt {
        println!("cargo:warning=   Using pre-built guest binaries");
        println!("cargo:warning=   All players will have IDENTICAL Image IDs");
        
        // Get OUT_DIR where methods.rs will be placed
        let out_dir = env::var("OUT_DIR").unwrap();
        let out_path = Path::new(&out_dir);
        
        // Copy methods.rs to OUT_DIR
        let dest_methods = out_path.join("methods.rs");
        fs::copy(prebuilt_methods, &dest_methods)
            .expect("Failed to copy prebuilt methods.rs");
        
        // Copy binaries to OUT_DIR
        // so that include_bytes!("init.bin") can find them
        for bin in &["init.bin", "round.bin"] {
            let src = Path::new("../prebuilt").join(bin);
            let dst = out_path.join(bin);
            fs::copy(&src, &dst)
                .expect(&format!("Failed to copy {}", bin));
        }
        
        println!("cargo:rerun-if-changed=../prebuilt/methods.rs");
        println!("cargo:rerun-if-changed=../prebuilt/init.bin");
        println!("cargo:rerun-if-changed=../prebuilt/round.bin");
    } else {
        println!("cargo:warning= Building from source");
        risc0_build::embed_methods();
    }
}