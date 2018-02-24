OUT_DIR=${PWD}/target/wasm32-unknown-unknown/debug/

all:
	@cargo +nightly build --target wasm32-unknown-unknown
	@cp index.html ${OUT_DIR}

http:
	@echo "open http://localhost:8000/index.html"
	@cd ${OUT_DIR} && python -m SimpleHTTPServer
