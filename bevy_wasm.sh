echo start
cargo build --example bevy_wasm --release --target wasm32-unknown-unknown
echo wasm-bindgen
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/examples/bevy_wasm.wasm
echo wasm-opt "(30s ?)"
wasm-opt -Oz out/bevy_wasm_bg.wasm --output out/better_bg.wasm
echo done
