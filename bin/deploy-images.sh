#!/bin/bash
# deploy-images.sh: Download OCI images from CI and load into K8s cluster
set -euo pipefail
trap 'echo "Deploy failed" >&2' ERR

REPO="chidionyema/ironcage"

echo "=== Ironcage CI Image Deployment ==="
echo "Downloading OCI image artifacts from latest CI build..."

RUN_ID=$(gh run list --repo "$REPO" --status success --json databaseId -q 2>/dev/null | head -1)
if [ -z "$RUN_ID" ]; then
	echo "ERROR: No successful CI runs" >&2
	exit 1
fi

echo "Latest run: $RUN_ID"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"; echo "Deploy failed" >&2' EXIT

cd "$TMPDIR"
gh run download "$RUN_ID" --repo "$REPO" --name ironcage-oci-images
ls -lh

echo "Loading images into Docker..."
docker load <ironcage-kernel.tar.gz
docker load <ironcage-api.tar.gz

docker images | grep ironcage

docker tag ironcage-kernel:v0.1.0 ironcage-kernel:latest
docker tag ironcage-api:v0.1.0 ironcage-api:latest

echo ""
echo "✅ Images loaded successfully"
echo ""
echo "Next: Push to registry or configure K8s node access"
echo "To push to Docker Hub:"
echo "  docker tag ironcage-kernel:latest YOUR_USER/ironcage-kernel:v0.1.0"
echo "  docker push YOUR_USER/ironcage-kernel:v0.1.0"
