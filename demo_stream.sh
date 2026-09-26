#!/bin/bash
# demo_stream.sh
# Generates a continuous stream of traffic to test PRISM TUI interactivity
# Usage: ./demo_stream.sh [PORT] (Default 514)

PORT=${1:-514}
echo "Starting continuous log stream to 127.0.0.1:$PORT... (Press Ctrl+C to stop)"

while true; do
  # Send 1000 normal logs
  for i in {1..1000}; do
    echo "logid=\"0000000013\" type=traffic srcip=192.168.1.$((RANDOM % 255))" > /dev/udp/127.0.0.1/$PORT
  done
  
  # Send 1 malformed log every ~1000 normal logs
  echo "192.168.1.$((RANDOM % 255)) - - [10/Oct/2026:13:55:36] \"GET /alien.html HTTP/1.1\" 404 0" > /dev/udp/127.0.0.1/$PORT
  
  # Small sleep to throttle slightly (so EPS stays around a readable 10k-20k instead of maxing CPU)
  sleep 0.1
done
