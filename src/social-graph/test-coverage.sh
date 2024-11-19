#!/bin/bash
set -e  # Exit immediately if a command exits with a non-zero status

# Create a directory to store coverage files
mkdir -p coverage

# Run tests and generate coverage files, excluding specific folders
for pkg in $(go list ./... | grep -vE "/mocks|/testutil"); do
    go test -covermode=atomic -coverpkg=./... -coverprofile=coverage/"$(echo "$pkg" | tr / -)" .out "$pkg"
done

# Merge all coverage profiles into a single profile
echo "mode: atomic" > coverage/coverage.out
grep -h -v "^mode:" coverage/*.out >> coverage/coverage.out

# Convert Go coverage to Cobertura format
gocover-cobertura < coverage/coverage.out > coverage/cobertura-coverage.xml
