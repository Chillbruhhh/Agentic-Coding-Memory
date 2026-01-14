#!/bin/bash

# Generate SDKs from OpenAPI specification

set -e

echo "=== Generating AMP SDKs ==="

# Check if openapi-generator is available
if ! command -v openapi-generator &> /dev/null; then
    echo "Installing openapi-generator..."
    npm install -g @openapitools/openapi-generator-cli
fi

SPEC_FILE="../spec/openapi.yaml"

# Generate Python SDK
echo "🐍 Generating Python SDK..."
mkdir -p sdks/python
openapi-generator generate \
    -i $SPEC_FILE \
    -g python \
    -o sdks/python \
    --package-name amp_client \
    --additional-properties=packageName=amp_client,projectName=amp-client

# Generate TypeScript SDK
echo "📜 Generating TypeScript SDK..."
mkdir -p sdks/typescript
openapi-generator generate \
    -i $SPEC_FILE \
    -g typescript-axios \
    -o sdks/typescript \
    --additional-properties=npmName=amp-client,npmVersion=0.1.0

echo "✅ SDKs generated successfully!"
echo "Python SDK: sdks/python/"
echo "TypeScript SDK: sdks/typescript/"
