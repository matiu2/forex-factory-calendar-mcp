#!/bin/bash
# Test the MCP server

# Use a FIFO (named pipe) to keep the connection open
FIFO_IN=$(mktemp -u)
FIFO_OUT=$(mktemp -u)
mkfifo "$FIFO_IN"
mkfifo "$FIFO_OUT"

# Start the MCP server (capture stderr too)
RUST_LOG=debug ~/.cargo/bin/forex-factory-calendar-mcp < "$FIFO_IN" > "$FIFO_OUT" 2>&1 &
SERVER_PID=$!

# Open the fifo for writing (keeps it open)
exec 3>"$FIFO_IN"

# Read responses in background
cat "$FIFO_OUT" &
CAT_PID=$!

# Send initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"0.1.0","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' >&3

sleep 0.5

# Send initialized notification
echo '{"jsonrpc":"2.0","method":"notifications/initialized"}' >&3

sleep 0.5

# Call get_week_events
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_week_events","arguments":{}}}' >&3

# Wait for response
sleep 30

# Cleanup
exec 3>&-
kill $SERVER_PID 2>/dev/null
kill $CAT_PID 2>/dev/null
rm -f "$FIFO_IN" "$FIFO_OUT"
