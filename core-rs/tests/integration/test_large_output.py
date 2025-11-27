# Test large output handling
import sys

for i in range(1000):
    print(f"Line {i}: " + "x" * 100)
    sys.stdout.flush()

print("Large output test complete")
