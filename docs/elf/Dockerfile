# We will use builder image to compile arwen and install it
FROM rust:1.67 as builder
WORKDIR /usr/src/arwen
# go up two directories to get the Arwen source code
COPY . .
RUN ls -la ../../
RUN ls -la /usr/src/arwen
RUN cargo install --path .



FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/arwen /usr/local/bin/arwen

RUN apt-get update && apt-get install -y --no-install-recommends \
    libc-bin \
    build-essential \
    binutils

# Create directory for C source code
WORKDIR /src

# Create sample library source code
RUN echo 'int custom_function() { return 42; }' > libcustom.c


# Create sample application source code
RUN echo '#include <stdio.h>\nint custom_function(); int main() { printf("Custom function returned: %d\\n", custom_function()); return 0; }' > my_app.c

ENV OUTPUT_DIR="/output"

RUN mkdir -p "$OUTPUT_DIR"

RUN gcc -shared -fPIC -o "$OUTPUT_DIR/libcustom.so" libcustom.c
RUN gcc my_app.c -o "$OUTPUT_DIR/my_app" -L"$OUTPUT_DIR" -lcustom



CMD ["bash"]
