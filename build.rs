use std::path::Path;
use std::process::Command;

fn auto_build_js_payloads() -> bool {
    let sources = [
        ("js/src/injected-core.ts", "js/dist/core.mjs"),
        ("js/src/injected-actions.ts", "js/dist/actions.mjs"),
    ];

    for &(src, dst) in &sources {
        let built = Command::new("vp")
            .args(["dlx", "pack"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || Command::new("npx")
                .args(["vp", "pack"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

        if !built {
            return false;
        }
    }

    sources.iter().all(|(_, dst)| Path::new(dst).exists())
}

fn main() {
    napi_build::setup();

    let core_src = Path::new("js/src/injected-core.ts");
    let actions_src = Path::new("js/src/injected-actions.ts");
    let core_dist = Path::new("js/dist/core.mjs");
    let actions_dist = Path::new("js/dist/actions.mjs");

    let dist_missing = !core_dist.exists() || !actions_dist.exists();
    let src_newer = core_src.exists()
        && actions_dist.exists()
        && (core_src.metadata().and_then(|m| m.modified()).ok()
            > core_dist.metadata().and_then(|m| m.modified()).ok()
            || actions_src.metadata().and_then(|m| m.modified()).ok()
                > actions_dist.metadata().and_then(|m| m.modified()).ok());

    if dist_missing || src_newer {
        println!("cargo:warning=Injected JS sources changed or missing, rebuilding...");
        if !auto_build_js_payloads() {
            panic!(
                "Failed to build injected JS payloads!\n\
                 Tried `vp dlx esbuild` and `npx esbuild` — neither succeeded.\n\
                 Run `vp run build:js` manually before building the Rust extension.\n\
                 Ensure esbuild is available (install via: npm install -g esbuild)."
            );
        }
        println!("cargo:warning=Injected JS payloads rebuilt successfully.");
    }

    if !core_dist.exists() || !actions_dist.exists() {
        panic!(
            "Injected JS payload not found!\n\
             Expected: {}\n\
             Expected: {}\n\
             Run 'vp run build:js' before building the Rust extension.",
            core_dist.display(),
            actions_dist.display()
        );
    }

    println!("cargo:rerun-if-changed=js/src/injected-core.ts");
    println!("cargo:rerun-if-changed=js/src/injected-actions.ts");
    println!("cargo:rerun-if-changed=js/dist/core.mjs");
    println!("cargo:rerun-if-changed=js/dist/actions.mjs");
}
