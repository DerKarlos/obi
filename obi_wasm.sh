echo start
cargo build --example obi_wasm --release --target wasm32-unknown-unknown
echo wasm-bindgen
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/examples/obi_wasm.wasm
echo wasm-opt "(30s ?)"
# rem
wasm-opt -Oz out/obi_wasm_bg.wasm --output out/obi_wasm_bg.wasm
echo done done done done done done done done done
