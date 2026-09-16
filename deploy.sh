#!/bin/bash
set -euo pipefail
trap 'echo "❌ Deployment failed" >&2' ERR

echo "🚀 Ironcage Deployment Script"
echo "=============================="

check_cmd() {
	command -v "$1" &>/dev/null
}

echo "✓ Checking prerequisites..."
check_cmd cargo || {
	echo "❌ cargo not found"
	exit 1
}
check_cmd kubectl || echo "⚠️  kubectl not found (skip K3s deployment)"

echo "✓ Building release binaries..."
cargo build --release

echo "✓ Running tests..."
cargo test --lib

if check_cmd docker; then
	echo "✓ Building Docker image..."
	docker build -t ironcage:latest .
fi

echo ""
echo "📊 Binary Sizes:"
ls -lh target/release/ironcage-* | awk '{print $9, "-", $5}'

if check_cmd kubectl && [[ -n "${DEPLOY_K3S:-}" ]]; then
	echo "✓ Deploying to K3s..."
	kubectl apply -f k8s/deployment.yaml
	kubectl wait --for=condition=ready pod -l app=ironcage-kernel -n ironcage --timeout=30s
	echo "✓ Kernel deployed and ready"
	echo "Access UI at: http://localhost:3001"
fi

echo ""
echo "✅ Ironcage ready!"
echo ""
echo "Run locally:"
echo "  cargo run --release --bin ironcage-kernel"
echo "  # In another terminal:"
echo "  cargo run --release --bin ironcage-api"
echo "  # Open browser to http://localhost:3001"
