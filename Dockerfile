# Use the official Rust image as base
FROM rust:1.86

# Install libxml2 development libraries and other dependencies
RUN apt update && apt install -y \
    lsb-release \
    wget \
    software-properties-common \
    gnupg \
    libxml2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN bash -c "$(wget -O - https://apt.llvm.org/llvm.sh)"

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

COPY src/ ./src/
COPY xsd-schemas/ ./xsd-schemas/

# Build the application
RUN cargo build --release

# Expose port 80
EXPOSE 80

# Run the application
CMD ["./target/release/hj2-api-xml-rust"]
