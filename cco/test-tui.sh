#!/bin/bash
echo "Testing CCO TUI startup..."
echo "=============================="
echo ""
echo "1. Checking if port 3000 is free..."
lsof -i:3000 2>/dev/null | grep LISTEN || echo "   ✓ Port 3000 is free"
echo ""

echo "2. Starting daemon in background..."
./target/release/cco run --port 3000 &
DAEMON_PID=$!
echo "   Daemon PID: $DAEMON_PID"

echo "3. Waiting for daemon to be ready..."
for i in {1..10}; do
    if curl -s http://127.0.0.1:3000/health > /dev/null 2>&1; then
        echo "   ✓ Daemon is ready after $i seconds"
        break
    fi
    sleep 1
done

echo ""
echo "4. Testing TUI connection to daemon..."
echo "   (Press 'q' to exit the TUI when it appears)"
echo ""

# Run TUI in foreground so we can see it
./target/release/cco

echo ""
echo "5. Cleaning up..."
kill $DAEMON_PID 2>/dev/null
echo "   ✓ Daemon stopped"
echo ""
echo "Test complete!"