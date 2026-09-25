#!/bin/bash

# Ensure we are in the right directory
cd /mnt/work/projects/sih/prism

echo "Starting PRISM natively..."
# Start prism, piping output to a log file
./target/release/prism -u 0.0.0.0:5514 --batch-size 10000 > prism_load.log 2>&1 &
PRISM_PID=$!

echo "PRISM running with PID $PRISM_PID. Waiting 5 seconds for initialization..."
sleep 5

echo "Starting load_gen..."
# Run the load generator
./target/release/load_gen
LOAD_GEN_EXIT=$?

echo "Load generator finished. Waiting 10 seconds for PRISM to flush queues..."
sleep 10

echo "Shutting down PRISM (SIGINT)..."
kill -2 $PRISM_PID
wait $PRISM_PID

echo "=========================================="
echo "PRISM FINAL STATS:"
echo "=========================================="
# Extract the last stats line
cat prism_load.log | grep "\[STATS\]" | tail -n 5

if [ $LOAD_GEN_EXIT -eq 0 ]; then
    echo "Performance test completed successfully."
else
    echo "Performance test failed."
fi
