#!/bin/bash
# demo_real_stream.sh
# Streams actual Fortinet, Cisco, and Palo Alto logs from the dataset directory

PORT=${1:-514}
echo "Starting REAL continuous log stream to 127.0.0.1:$PORT... (Press Ctrl+C to stop)"

# Pre-load logs into memory for speed
FORTINET_LOG=$(head -n 1 data/samples/fortinet_fortigate.log)
CISCO_LOG=$(head -n 1 data/samples/cisco_asa.log)
PALOALTO_LOG=$(head -n 1 data/samples/paloalto_threat.log)

while true; do
  for i in {1..200}; do
    # Fortinet
    echo "$FORTINET_LOG" > /dev/udp/127.0.0.1/$PORT
    # Cisco
    echo "$CISCO_LOG" > /dev/udp/127.0.0.1/$PORT
    # Palo Alto
    echo "$PALOALTO_LOG" > /dev/udp/127.0.0.1/$PORT
  done
  
  # Send an anomaly (Alien log)
  echo "192.168.1.$((RANDOM % 255)) - - [10/Oct/2026:13:55:36] \"GET /admin_shell.php HTTP/1.1\" 404 0" > /dev/udp/127.0.0.1/$PORT
  
  sleep 0.1
done
