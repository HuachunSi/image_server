# Use the official Rust 1.85 image as the base
FROM rust:1.85-bookworm as base

# Install tools for building native binaries
RUN sed -i 's@deb.debian.org@mirrors.aliyun.com@g' /etc/apt/sources.list.d/debian.sources
# RUN echo "deb http://mirrors.aliyun.com/debian testing main" > /etc/apt/sources.list 
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       pkg-config \
       libseccomp-dev \
       wget \
       curl \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*


# Replace the default Cargo config with the sproxy-sparse
COPY .cargo ./.cargo

# Set the working directory in the container
WORKDIR /usr/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Expose the port the app runs on
EXPOSE 7870

# # Copy the compiled binary to the container
# RUN mv target/release/image_server /usr/bin/image_server && chmod +x /usr/bin/image_server && rm -r target

# # Set the entry point to run the application
# CMD ["image_server"]

FROM --platform=linux/amd64 nginx:latest as production

WORKDIR /usr/app

COPY --from=base /usr/app/target/release/image_server /usr/bin/image_server

RUN chmod +x /usr/bin/image_server

CMD ["image_server"]

