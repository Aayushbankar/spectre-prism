#!/bin/bash
echo "=========================================="
echo " PRISM EDGE CASE TRAFFIC INJECTOR"
echo "=========================================="

echo "[*] Test 1: Sending Known Hardware Log (Palo Alto via UDP)"
echo '1,2026/09/26 10:00:00,010108010441,TRAFFIC,start,2305,2026/09/26 10:00:00,10.10.20.45,203.0.113.25,0.0.0.0,0.0.0.0,Rule-Web,user1,,ssl,vsys1,Trust,Untrust,ethernet1/2,ethernet1/1,LogForwarding_Syslog,2026/09/26 10:00:00,654123,1,52341,443,0,0,0x0,tcp,allow,1420,700,720,12,6,6,0' > /dev/udp/127.0.0.1/5514
sleep 3

echo "[*] Test 2: Sending 3 unique unknown NGINX logs (Drain3 Dedup Test)"
echo '192.168.1.55 - - [10/Oct/2026:13:00:00] "GET /test1 HTTP/1.1" 404 123' > /dev/udp/127.0.0.1/5514
echo '10.0.0.99 - - [11/Oct/2026:14:15:22] "GET /test2 HTTP/1.1" 404 456' > /dev/udp/127.0.0.1/5514
echo '172.16.5.4 - - [12/Oct/2026:15:30:11] "GET /test3 HTTP/1.1" 404 789' > /dev/udp/127.0.0.1/5514

echo ""
echo "[!] ACTION REQUIRED: Check AI Brain terminal to see it parse ONLY 1 rule."
echo "[!] ACTION REQUIRED: Go to TUI -> Gatekeeper Tab -> Press 'Enter' to Approve the rule!"
echo "Waiting 15 seconds for you to manually approve..."
sleep 15
echo ""

echo "[*] Test 3: Sending TCP Stream Log (Universal Ingest Test)"
# Should parse instantly because you just approved the rule
echo '10.0.0.200 - - [12/Oct/2026:16:00:00] "GET /tcp-stream HTTP/1.1" 200 999' > /dev/tcp/127.0.0.1/6514
sleep 3

echo "[*] Test 4: Sending Garbage Data (Fuzzing / Error Handling)"
head -c 100 /dev/urandom > /dev/udp/127.0.0.1/5514
sleep 3

echo "[*] Test 5: Triggering Continuous Load Stream (TUI EPS Scale Test)..."
./demo_real_stream.sh 5514 > /dev/null 2>&1 &
STREAM_PID=$!
echo "Streaming at max speed for 10 seconds..."
sleep 10
kill -9 $STREAM_PID 2>/dev/null || true
echo "[*] Stream stopped. All tests completed!"
