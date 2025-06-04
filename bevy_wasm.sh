cargo build --example bevy_wasm --release --target wasm32-unknown-unknown
echo wasm-bindgen
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/examples/bevy_wasm.wasm
