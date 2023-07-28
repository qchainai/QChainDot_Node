# Note: This is currently designed to simplify development
# To get a smaller docker image, there should be 2 images generated, in 2 stages.

FROM rustlang/rust:nightly

ARG PROFILE=release
WORKDIR /node

RUN apt-get install -y openssh-client git

RUN mkdir -p -m 0700 ~/.ssh && ssh-keyscan bitbucket.org >> ~/.ssh/known_hosts

# Upcd dates core parts
RUN apt-get update -y && \
	apt-get install -y cmake pkg-config libssl-dev git gcc build-essential clang libclang-dev protobuf-compiler

# Install rust wasm. Needed for substrate wasm engine
RUN rustup target add wasm32-unknown-unknown

# Download Frontier repo
RUN git clone https://loviglass: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@github.com/loviglass/node /node
RUN #--mount=type=ssh git clone git clone git@bitbucket.org:qchainai/node /node
RUN cd /node && git submodule init && git submodule update

# Download rust dependencies and build the rust binary
RUN --mount=type=ssh cargo build --config net.git-fetch-with-cli=true "--$PROFILE"

# 30333 for p2p traffic
# 9933 for RPC call
# 9944 for Websocket
# 9615 for Prometheus (metrics)
EXPOSE 30333 9933 9944 9615


ENV PROFILE ${PROFILE}

# The execution will re-compile the project to run it
# This allows to modify the code and not have to re-compile the
# dependencies.
#CMD cargo run --bin qchain-template-node "--$PROFILE" -- --dev
CMD cargo run --bin qchain-template-node "--$PROFILE"