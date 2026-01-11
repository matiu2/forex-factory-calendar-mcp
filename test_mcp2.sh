#!/bin/bash
# Test the MCP server - debug build

FIFO_IN=$(mktemp -u)
FIFO_OUT=$(mktemp -u)
mkfifo "$FIFO_IN"
mkfifo "$FIFO_OUT"

# Start the debug MCP server
RUST_LOG=debug ./target/debug/forex-factory-calendar-mcp < "$FIFO_IN" > "$FIFO_OUT" 2>&1 &
SERVER_PID=$!

exec 3>"$FIFO_IN"
cat "$FIFO_OUT" &
CAT_PID=$!

echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' >&3
sleep 0.5
echo '{"jsonrpc":"2.0","method":"notifications/initialized"}' >&3
sleep 0.5
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_week_events","arguments":{}}}' >&3
sleep 30

exec 3>&-
kill $SERVER_PID 2>/dev/null
kill $CAT_PID 2>/dev/null
rm -f "$FIFO_IN" "$FIFO_OUT"
