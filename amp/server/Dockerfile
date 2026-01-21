FROM rust:latest

# Install dependencies for RocksDB and bindgen
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    llvm-dev \
    cmake \
    build-essential \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Set libclang path - find the actual location dynamically
RUN LIBCLANG_DIR=$(find /usr/lib -name "libclang.so*" -exec dirname {} \; | head -1) && \
    echo "export LIBCLANG_PATH=$LIBCLANG_DIR" >> /etc/profile.d/libclang.sh

# Set environment variables for bindgen
ENV LIBCLANG_PATH=/usr/lib/x86_64-linux-gnu

CMD ["cargo", "run", "-p", "amp-server"]
