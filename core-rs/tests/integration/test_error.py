# Test error handling
import sys

print("Testing error output")
sys.stderr.write("This is an error message\n")

# Intentional error
raise ValueError("Test error message for verification")
