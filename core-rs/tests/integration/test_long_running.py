# Test long-running script (for stop test)
import time
import sys

print("Starting long running script...")
sys.stdout.flush()

for i in range(60):
    print(f"Running... {i}/60 seconds")
    sys.stdout.flush()
    time.sleep(1)

print("Script completed")
