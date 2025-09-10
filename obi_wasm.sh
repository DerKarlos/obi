echo start
cargo build --example obi_wasm --release --target wasm32-unknown-unknown
echo wasm-bindgen
wasm-bindgen --out-dir ./obi/ --target web ./target/wasm32-unknown-unknown/release/examples/obi_wasm.wasm
echo wasm-opt "(30s ?)"
# rem
wasm-opt -Oz obi/obi_wasm_bg.wasm --output obi/obi_wasm_bg.wasm
echo done done done done done done done done done

# For wasm, you should use wasm-opt to reduce the size,
# possibly along with compilation flags like opt-level='z', lto=true and codegen-units=1.
# Note that you should only use these for release builds for wasm.
# Debug builds will be way larger anyways, and you should probably develop/debug with a native build.
