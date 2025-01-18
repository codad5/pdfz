# Use the Rust image with all required tools
FROM rust:1.81

# Install necessary system dependencies
RUN apt-get update && \
    apt-get install -y \
    tesseract-ocr \
    libtesseract-dev \
    libleptonica-dev \
    tesseract-ocr-eng \
    libicu-dev \
    libpango1.0-dev \
    libcairo2-dev \
    jq \
    git \
    python3-pip \
    python3-venv \
    make \
    unzip \
    nano \
    imagemagick \
    protobuf-compiler \
    && apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Install cargo-watch for live reloading
RUN cargo install cargo-watch

# Set default working directory
WORKDIR /extractor

# Copy only Cargo files to leverage Docker's layer caching for dependencies
COPY Cargo.toml Cargo.lock ./

# Pre-build dependencies to cache them
RUN cargo build --release || true

# Copy the rest of the application code
COPY . .

# Set up Python virtual environment for tesstrain if required
RUN git clone https://github.com/tesseract-ocr/tesstrain.git /tesstrain && \
    python3 -m venv /tesstrain/venv && \
    /tesstrain/venv/bin/pip install --upgrade pip && \
    /tesstrain/venv/bin/pip install -r /tesstrain/requirements.txt

# Add tools to the PATH
ENV PATH="/tesstrain:/tesstrain/venv/bin:/usr/local/cargo/bin:$PATH"

# Expose the gRPC port
EXPOSE 50051

# Run the app with live reloading
CMD ["cargo", "watch", "-x", "run"]

# FROM rust:1.81 AS builder

# # Install necessary dependencies
# RUN apt-get update && \
#     apt-get install -y \
#     tesseract-ocr \
#     libtesseract-dev \
#     libleptonica-dev \
#     tesseract-ocr-eng \
#     libicu-dev \
#     libpango1.0-dev \
#     libcairo2-dev \
#     jq \
#     git \
#     python3-pip \
#     python3-venv \
#     make \
#     unzip \
#     nano \
#     imagemagick \
#     protobuf-compiler \
#     && apt-get clean && \
#     rm -rf /var/lib/apt/lists/*

# # Clone the tesstrain repository
# RUN git clone https://github.com/tesseract-ocr/tesstrain.git /tesstrain

# # Set up Python virtual environment
# RUN python3 -m venv /tesstrain/venv

# # Activate the virtual environment and install requirements
# RUN /tesstrain/venv/bin/pip install --upgrade pip && \
#     /tesstrain/venv/bin/pip install -r /tesstrain/requirements.txt

# # Add training tools to PATH
# ENV PATH="/tesstrain:/tesstrain/venv/bin:$PATH"

# # Set default user
# ENV USER=codad5

# WORKDIR /extractor

# # We'll copy the proto file during build
# COPY ./proto /extractor/proto

# # Copy the rest of the application
# COPY . .

# RUN cargo build --release

# FROM debian:bookworm

# WORKDIR /extractor
# COPY --from=builder /extractor/target/release/pdfz .

# # Install required dependencies
# RUN apt-get update && apt-get install -y libc6 && rm -rf /var/lib/apt/lists/*


# EXPOSE 50051

# CMD ["cargo", "watch", "-x", "run"]



# FROM rust:1.81

# # Install necessary dependencies
# RUN apt-get update && \
#     apt-get install -y \
#     tesseract-ocr \
#     libtesseract-dev \
#     libleptonica-dev \
#     tesseract-ocr-eng \
#     libicu-dev \
#     libpango1.0-dev \
#     libcairo2-dev \
#     jq \
#     git \
#     python3-pip \
#     python3-venv \
#     make \
#     unzip \
#     nano \
#     imagemagick \
#     && apt-get clean && \
#     rm -rf /var/lib/apt/lists/*

# # Clone the tesstrain repository
# RUN git clone https://github.com/tesseract-ocr/tesstrain.git /tesstrain

# # Set up Python virtual environment
# RUN python3 -m venv /tesstrain/venv

# # Activate the virtual environment and install requirements
# RUN /tesstrain/venv/bin/pip install --upgrade pip && \
#     /tesstrain/venv/bin/pip install -r /tesstrain/requirements.txt

# # Add training tools to PATH
# ENV PATH="/tesstrain:/tesstrain/venv/bin:$PATH"

# # set default user
# ENV USER=codad5

# # Set the working directory
# WORKDIR /app
