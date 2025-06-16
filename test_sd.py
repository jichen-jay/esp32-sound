#!/usr/bin/env python3
"""
T-Embed SD Card Troubleshooting Helper
This script helps diagnose SD card issues with the T-Embed
"""

import time

def print_troubleshooting_guide():
    print("=== T-Embed SD Card Troubleshooting Guide ===")
    print()
    print("ERROR: ESP_ERR_INVALID_RESPONSE (0x108)")
    print("This usually means the SD card is not responding properly.")
    print()
    
    print("1. HARDWARE CHECKS:")
    print("   - Ensure SD card is properly inserted")
    print("   - Check that SD card is not write-protected (switch position)")
    print("   - Verify SD card is working (test in computer)")
    print("   - Try a different SD card (some cards are incompatible)")
    print()
    
    print("2. WIRING VERIFICATION (T-Embed pins):")
    print("   CS   -> GPIO13")
    print("   MOSI -> GPIO9")
    print("   MISO -> GPIO10") 
    print("   CLK  -> GPIO11")
    print("   VCC  -> 3.3V")
    print("   GND  -> GND")
    print()
    
    print("3. SD CARD COMPATIBILITY:")
    print("   - Use SD cards 32GB or smaller")
    print("   - Format as FAT32 (not exFAT)")
    print("   - Avoid high-speed cards (Class 10+ sometimes problematic)")
    print("   - Try older/slower SD cards")
    print()
    
    print("4. POWER SUPPLY:")
    print("   - Ensure stable 3.3V supply")
    print("   - Check for voltage drops under load")
    print("   - Try with external power supply")
    print()
    
    print("5. SOFTWARE SOLUTIONS:")
    print("   - Lower SPI frequency (already implemented)")
    print("   - Disable pull-ups if external pull-ups present")
    print("   - Try different SPI modes")
    print()
    
    print("6. RECOMMENDED SD CARDS FOR TESTING:")
    print("   - SanDisk 8GB or 16GB (older models)")
    print("   - Kingston 8GB Class 4")
    print("   - Generic 4GB-8GB cards")
    print()
    
    print("Try the updated code with slower SPI speeds and better error handling.")

if __name__ == "__main__":
    print_troubleshooting_guide()