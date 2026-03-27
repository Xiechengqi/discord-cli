#!/bin/bash
set -e

echo "Building discord-cli frontend..."
(cd frontend && npm install && npm run build)

echo "Building discord-cli binary..."
cargo build --release

echo "Build complete: target/release/discord-cli"
