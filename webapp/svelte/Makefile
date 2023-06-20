all: webapp wasm

webapp: wasm
	npm ci
	npm run build

wasm:
	cd wasm-sample && wasm-pack build --target web

fmt:
	cd wasm-sample && cargo fmt
	npm run format
