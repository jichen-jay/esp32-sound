#!/bin/bash

echo "Fixing T-Embed ESP32-S3 build configuration..."

# Clean previous build
echo "Cleaning previous build..."
idf.py clean

# Set target to ESP32-S3
echo "Setting target to ESP32-S3..."
idf.py set-target esp32s3

# Remove any existing sdkconfig to force regeneration
echo "Removing existing sdkconfig..."
rm -f sdkconfig

# Copy our defaults
echo "Applying T-Embed configuration..."
cp sdkconfig.defaults sdkconfig

# Build the project
echo "Building project..."
idf.py build

echo "Build fix complete. Try flashing with:"
echo "idf.py flash monitor"