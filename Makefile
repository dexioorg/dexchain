.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run
run:
	 cargo run --release -- --dev --tmp

.PHONY: build
build:
	 cargo build --release

.PHONY: build-runtime
build-runtime:
	 cargo build --release -p dexchain-runtime

.PHONY: build-spec
build-spec:
	./target/release/dexchain build-spec --disable-default-bootnode --chain dexchain-staging > testnet.json

.PHONY: build-spec-raw
build-spec-raw:
	./target/release/dexchain build-spec --chain=testnet.json --raw --disable-default-bootnode > testnetRaw.json

.PHONY: benchmark
benchmark:
	cargo run --release --bin dexchain --features runtime-benchmarks -- benchmark \
	--chain dev \
	--execution wasm \
	--wasm-execution Interpreted \
	--pallet=* \
	--extrinsic=* \
	--steps 50 \
	--repeat 20 \
	--heap-pages 4096 \
	--raw \
	--output ./runtime-weights/ \
	--template ./templates/runtime-weight-template.hbs