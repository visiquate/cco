#!/bin/bash
# XOR obfuscate the orchestrator prompt at build time
# This provides basic protection from casual inspection

INPUT="config/orchestrator-prompt.txt"
OUTPUT="config/orchestrator-prompt.bin"
XOR_KEY=167  # 0xA7 in decimal

if [ ! -f "$INPUT" ]; then
    echo "Error: $INPUT not found"
    exit 1
fi

# Use Python for reliable cross-platform XOR obfuscation
python3 << 'EOF'
import sys

try:
    with open("config/orchestrator-prompt.txt", "rb") as f:
        data = f.read()

    XOR_KEY = 167
    obfuscated = bytes([b ^ XOR_KEY for b in data])

    with open("config/orchestrator-prompt.bin", "wb") as f:
        f.write(obfuscated)

    print(f"Obfuscated {len(data)} bytes → config/orchestrator-prompt.bin")
    sys.exit(0)
except Exception as e:
    print(f"Error: {e}", file=sys.stderr)
    sys.exit(1)
EOF
