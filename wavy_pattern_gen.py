#!/usr/bin/env python3
"""
Visual Pattern Generator for ESP32 I2S Oscilloscope Display
Creates custom patterns that look good on oscilloscope screens
"""

import math
import matplotlib.pyplot as plt
import numpy as np

def generate_square_wave(samples=64, amplitude=0x8000, duty_cycle=0.5):
    """Generate a square wave pattern"""
    pattern = []
    high_samples = int(samples * duty_cycle)
    
    for i in range(samples):
        if i < high_samples:
            pattern.append(amplitude)
        else:
            pattern.append(0x0000)
    
    return pattern

def generate_triangle_wave(samples=64, amplitude=0x8000):
    """Generate a triangle wave pattern"""
    pattern = []
    half_samples = samples // 2
    
    for i in range(samples):
        if i < half_samples:
            # Rising edge
            value = int((i / half_samples) * amplitude)
        else:
            # Falling edge
            value = int(((samples - i) / half_samples) * amplitude)
        pattern.append(value)
    
    return pattern

def generate_sawtooth_wave(samples=64, amplitude=0x8000):
    """Generate a sawtooth wave pattern"""
    pattern = []
    
    for i in range(samples):
        value = int((i / samples) * amplitude)
        pattern.append(value)
    
    return pattern

def generate_sine_wave(samples=64, amplitude=0x4000, cycles=2):
    """Generate a sine wave pattern"""
    pattern = []
    
    for i in range(samples):
        angle = (i / samples) * 2 * math.pi * cycles
        value = int(amplitude + amplitude * math.sin(angle))
        pattern.append(max(0, min(0xFFFF, value)))
    
    return pattern

def generate_staircase(samples=64, steps=8, amplitude=0x8000):
    """Generate a staircase pattern"""
    pattern = []
    samples_per_step = samples // steps
    
    for step in range(steps):
        step_value = int((step / (steps - 1)) * amplitude)
        for _ in range(samples_per_step):
            pattern.append(step_value)
    
    # Fill remaining samples
    while len(pattern) < samples:
        pattern.append(amplitude)
    
    return pattern[:samples]

def generate_heart_shape(samples=64, amplitude=0x8000):
    """Generate a heart-shaped pattern"""
    pattern = []
    
    for i in range(samples):
        t = (i / samples) * 4 * math.pi  # Two full cycles
        
        # Heart equation (simplified for oscilloscope)
        # x = 16sin³(t), y = 13cos(t) - 5cos(2t) - 2cos(3t) - cos(4t)
        heart_y = 13 * math.cos(t) - 5 * math.cos(2*t) - 2 * math.cos(3*t) - math.cos(4*t)
        
        # Normalize and scale
        normalized = (heart_y + 21) / 42  # Normalize to 0-1
        value = int(normalized * amplitude)
        pattern.append(max(0, min(0xFFFF, value)))
    
    return pattern

def generate_house_pattern(samples=64, amplitude=0x8000):
    """Generate a house-shaped pattern"""
    pattern = []
    sections = samples // 8
    
    # Foundation
    for _ in range(sections):
        pattern.append(int(0.2 * amplitude))
    
    # Left wall rising
    for i in range(sections):
        value = int((0.2 + 0.4 * (i / sections)) * amplitude)
        pattern.append(value)
    
    # Roof peak
    for i in range(sections):
        if i < sections // 2:
            value = int((0.6 + 0.4 * (i / (sections // 2))) * amplitude)
        else:
            value = int((1.0 - 0.4 * ((i - sections // 2) / (sections // 2))) * amplitude)
        pattern.append(value)
    
    # Right wall down
    for i in range(sections):
        value = int((0.6 - 0.4 * (i / sections)) * amplitude)
        pattern.append(value)
    
    # Foundation again
    for _ in range(sections):
        pattern.append(int(0.2 * amplitude))
    
    # Door
    for i in range(sections):
        if i < sections // 3 or i > 2 * sections // 3:
            pattern.append(int(0.2 * amplitude))
        else:
            pattern.append(int(0.35 * amplitude))
    
    # Window
    for i in range(sections):
        if i < sections // 4 or i > 3 * sections // 4:
            pattern.append(int(0.2 * amplitude))
        else:
            pattern.append(int(0.5 * amplitude))
    
    # Final foundation
    for _ in range(sections):
        pattern.append(int(0.2 * amplitude))
    
    return pattern[:samples]

def generate_custom_text_pattern(text="HI", samples=64, amplitude=0x8000):
    """Generate a pattern that spells out text (simplified)"""
    pattern = []
    
    if text == "HI":
        # H pattern
        quarter = samples // 4
        
        # Left vertical line of H
        for i in range(quarter):
            pattern.append(int(0.8 * amplitude))
        
        # Horizontal line of H  
        for i in range(quarter):
            value = int((0.4 + 0.4 * (i / quarter)) * amplitude)
            pattern.append(value)
        
        # Right vertical line of H
        for i in range(quarter):
            pattern.append(int(0.8 * amplitude))
        
        # Space, then I
        for i in range(quarter):
            if i < quarter // 4 or i > 3 * quarter // 4:
                pattern.append(int(0.1 * amplitude))  # Low for space
            else:
                pattern.append(int(0.7 * amplitude))  # High for I
    
    return pattern

def plot_pattern(pattern, title="Pattern"):
    """Plot the pattern for visualization"""
    plt.figure(figsize=(12, 6))
    plt.plot(pattern, linewidth=2)
    plt.title(f"{title} - {len(pattern)} samples")
    plt.xlabel("Sample Index")
    plt.ylabel("Amplitude (16-bit)")
    plt.grid(True, alpha=0.3)
    plt.ylim(0, 0x8000)
    
    # Add hex values for easy copying
    hex_values = [f"0x{val:04x}" for val in pattern[:16]]
    plt.figtext(0.02, 0.02, f"First 16 values: {', '.join(hex_values)}", fontsize=8)
    
    plt.tight_layout()
    plt.show()

def pattern_to_rust_array(pattern, name="CUSTOM_PATTERN"):
    """Convert pattern to Rust array format"""
    rust_code = f"const {name}: &[u16] = &[\n"
    
    for i in range(0, len(pattern), 8):
        line = "    "
        for j in range(8):
            if i + j < len(pattern):
                line += f"0x{pattern[i + j]:04x}"
                if i + j < len(pattern) - 1:
                    line += ", "
        line += "\n"
        rust_code += line
    
    rust_code += "];\n"
    return rust_code

def create_story_patterns():
    """Create a series of patterns that tell a visual story"""
    
    print("🎨 Creating Visual Story Patterns for Oscilloscope 🎨\n")
    
    # Story: Building a house, adding a heart, making people smile
    patterns = {
        "Foundation": generate_staircase(64, 4, 0x3000),
        "Building_Walls": generate_triangle_wave(64, 0x6000),
        "Adding_Roof": generate_house_pattern(64, 0x8000),
        "Adding_Love": generate_heart_shape(64, 0x7000),
        "Happy_Ending": generate_sine_wave(64, 0x4000, 1),  # Gentle smile
        "Celebration": generate_square_wave(64, 0x8000, 0.3),  # Party lights
    }
    
    print("📖 Story Pattern Sequence:")
    for i, (name, pattern) in enumerate(patterns.items(), 1):
        print(f"{i}. {name}: {len(pattern)} samples")
        
        # Show first few values
        preview = [f"0x{val:04x}" for val in pattern[:8]]
        print(f"   Preview: {', '.join(preview)}...")
        print()
    
    # Generate complete Rust code
    rust_code = "// Visual Story Patterns for ESP32 I2S Oscilloscope Display\n\n"
    
    for name, pattern in patterns.items():
        rust_code += pattern_to_rust_array(pattern, f"{name.upper()}_PATTERN")
        rust_code += "\n"
    
    # Create pattern array
    rust_code += "const STORY_PATTERNS: &[&[u16]] = &[\n"
    for name in patterns.keys():
        rust_code += f"    &{name.upper()}_PATTERN,\n"
    rust_code += "];\n\n"
    
    rust_code += 'const STORY_NAMES: &[&str] = &[\n'
    for name in patterns.keys():
        rust_code += f'    "{name}",\n'
    rust_code += "];\n"
    
    # Save to file
    with open("story_patterns.rs", "w") as f:
        f.write(rust_code)
    
    print("📝 Generated story_patterns.rs with complete Rust code!")
    print("🎯 Copy this into your ESP32 project for a visual story sequence")
    
    return patterns

def generate_oscilloscope_test_patterns():
    """Generate patterns specifically designed for oscilloscope testing"""
    
    test_patterns = {
        "Calibration_Square": generate_square_wave(32, 0x8000, 0.5),
        "Frequency_Test": generate_sine_wave(64, 0x4000, 4),
        "Amplitude_Steps": generate_staircase(48, 6, 0x8000),
        "Trigger_Test": [0x0000] * 16 + [0x8000] * 16 + [0x0000] * 16 + [0x8000] * 16,
        "Noise_Pattern": [0x4000 + int(0x1000 * math.sin(i * 0.5)) for i in range(64)],
    }
    
    print("\n🔬 Oscilloscope Test Patterns:")
    for name, pattern in test_patterns.items():
        print(f"✅ {name}: {len(pattern)} samples")
    
    return test_patterns

def main():
    """Main function to generate and display patterns"""
    
    print("🎨 ESP32 I2S Visual Pattern Generator 🎨")
    print("=" * 50)
    
    # Create basic patterns
    patterns = {
        "Square Wave": generate_square_wave(64),
        "Triangle Wave": generate_triangle_wave(64),
        "Sawtooth Wave": generate_sawtooth_wave(64),
        "Sine Wave": generate_sine_wave(64, 0x4000, 2),
        "Staircase": generate_staircase(64, 8),
        "Heart Shape": generate_heart_shape(64),
        "House Pattern": generate_house_pattern(64),
    }
    
    print("\n📊 Generated Basic Patterns:")
    for name, pattern in patterns.items():
        print(f"✅ {name}: {len(pattern)} samples")
        
        # Show statistics
        min_val = min(pattern)
        max_val = max(pattern)
        avg_val = sum(pattern) // len(pattern)
        print(f"   Range: 0x{min_val:04x} - 0x{max_val:04x}, Avg: 0x{avg_val:04x}")
    
    # Generate story patterns
    story_patterns = create_story_patterns()
    
    # Generate test patterns
    test_patterns = generate_oscilloscope_test_patterns()
    
    print("\n🎯 Usage Instructions:")
    print("1. Copy the generated Rust arrays into your ESP32 code")
    print("2. Connect oscilloscope to GPIO6 (DOUT)")
    print("3. Set trigger to rising edge")
    print("4. Adjust time scale to ~100μs-1ms per division")
    print("5. Adjust voltage scale to see full pattern range")
    print("6. Look for recognizable shapes!")
    
    print("\n📈 Recommended Oscilloscope Settings:")
    print("• Trigger: Rising edge, ~1.65V threshold")
    print("• Time/div: 100μs - 1ms (depending on sample rate)")
    print("• Voltage/div: 500mV - 1V")
    print("• Coupling: DC")
    print("• Bandwidth: 20MHz or higher")
    
    # Plot a few patterns for visualization
    try:
        import matplotlib.pyplot as plt
        
        print("\n📊 Plotting sample patterns...")
        
        fig, axes = plt.subplots(2, 2, figsize=(15, 10))
        fig.suptitle("ESP32 I2S Visual Patterns for Oscilloscope", fontsize=16)
        
        # Plot key patterns
        patterns_to_plot = [
            ("Square Wave", patterns["Square Wave"]),
            ("Heart Shape", patterns["Heart Shape"]),
            ("House Pattern", patterns["House Pattern"]),
            ("Staircase", patterns["Staircase"])
        ]
        
        for i, (name, pattern) in enumerate(patterns_to_plot):
            row, col = i // 2, i % 2
            axes[row, col].plot(pattern, linewidth=2, marker='o', markersize=3)
            axes[row, col].set_title(name)
            axes[row, col].set_xlabel("Sample Index")
            axes[row, col].set_ylabel("Amplitude")
            axes[row, col].grid(True, alpha=0.3)
            axes[row, col].set_ylim(0, 0x8000)
        
        plt.tight_layout()
        plt.savefig("i2s_visual_patterns.png", dpi=300, bbox_inches='tight')
        print("✅ Saved pattern visualization as 'i2s_visual_patterns.png'")
        
    except ImportError:
        print("📊 Install matplotlib to see pattern visualizations")
    
    print("\n🚀 Ready to create amazing oscilloscope visuals!")

if __name__ == "__main__":
    main()