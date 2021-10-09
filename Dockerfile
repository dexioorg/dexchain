# Note: This is currently designed to simplify development
# To get a smaller docker image, there should be 2 images generated, in 2 stages.

FROM rustlang/rust:nightly


ARG PROFILE=release
WORKDIR /dexchain

# Upcd dates core parts
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git build-essential clang libclang-dev curl

# Install rust wasm. Needed for substrate wasm engine
RUN rustup toolchain install nightly-2021-01-13
RUN rustup target add wasm32-unknown-unknown --toolchain nightly-2021-01-13

# Download dexchain repo
RUN git clone https://github.com/dexioorg/dexchain /dexchain
RUN cd /dexchain && git submodule init && git submodule update

# Initializing WASM build environment
# RUN ./scripts/init.sh

# Download rust dependencies and build the rust binary
RUN cargo build "--$PROFILE"

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615


ENV PROFILE ${PROFILE}

# The execution will re-compile the project to run it
# This allows to modify the code and not have to re-compile the
# dependencies.
CMD cargo run --bin dexchain "--$PROFILE" -- --dev
