# TurboSheet - Full browser support
# Multi-stage build for minimal image size
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build TurboSheet
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release && \
    strip target/release/libturbosheet.so

# Production image with all browsers
FROM ubuntu:22.04

# Install system dependencies and browsers
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    gnupg \
    curl \
    ca-certificates \
    fonts-liberation \
    libasound2 \
    libatk-bridge2.0-0 \
    libatk1.0-0 \
    libcups2 \
    libdbus-1-3 \
    libdrm2 \
    libgbm1 \
    libgtk-3-0 \
    libnspr4 \
    libnss3 \
    libx11-xcb1 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    xdg-utils \
    && rm -rf /var/lib/apt/lists/*

# Chromium
RUN wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | apt-key add - && \
    echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" > /etc/apt/sources.list.d/google-chrome.list && \
    apt-get update && apt-get install -y google-chrome-stable && \
    rm -rf /var/lib/apt/lists/*

# Firefox (via apt)
RUN apt-get update && apt-get install -y firefox-esr && \
    rm -rf /var/lib/apt/lists/*

# WebKit
RUN apt-get update && apt-get install -y \
    libwebkit2gtk-4.0-37 \
    && rm -rf /var/lib/apt/lists/*

# Copy TurboSheet binary
COPY --from=builder /app/target/release/libturbosheet.so /usr/local/lib/
COPY --from=builder /app/index.js /usr/local/lib/
COPY --from=builder /app/index.d.ts /usr/local/lib/

# Install TurboSheet CLI
RUN npm install -g /usr/local/lib/

WORKDIR /workspace
ENTRYPOINT ["tsheet"]
CMD ["test"]
