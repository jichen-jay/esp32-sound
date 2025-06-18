//! ESP32-H2 I2S Visual Pattern Generator
//! Creates oscilloscope-friendly visual patterns instead of audio
//! GPIO4: BCLK, GPIO5: WS, GPIO6: DOUT

#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_buffers,
    gpio::{Io, Level, Output},
    i2s::{DataFormat, I2s, I2sWrite, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;
use esp_backtrace as _;
use esp_hal::entry;

// Visual pattern data for oscilloscope viewing
// Each pattern creates distinct shapes when viewed on oscilloscope
const VISUAL_PATTERNS: &[&[u16]] = &[
    &SQUARE_WAVE_PATTERN,
    &TRIANGLE_WAVE_PATTERN,
    &SAWTOOTH_PATTERN,
    &STAIRCASE_PATTERN,
    &HEART_SHAPE_PATTERN,
    &HOUSE_PATTERN,
    &SMILEY_FACE_PATTERN,
];

// Pattern 1: Square Wave - Clean rectangular pulses
const SQUARE_WAVE_PATTERN: &[u16] = &[
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, // High
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, // Low
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, // High
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000,
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, // Low
    0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
];

// Pattern 2: Triangle Wave - Smooth ramps up and down
const TRIANGLE_WAVE_PATTERN: &[u16] = &[
    0x0000, 0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, // Rising
    0x8000, 0x7000, 0x6000, 0x5000, 0x4000, 0x3000, 0x2000, 0x1000, // Falling
    0x0000, 0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, // Rising
    0x8000, 0x7000, 0x6000, 0x5000, 0x4000, 0x3000, 0x2000, 0x1000, // Falling
    0x0000, 0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, // Rising
    0x8000, 0x7000, 0x6000, 0x5000, 0x4000, 0x3000, 0x2000, 0x1000, // Falling
    0x0000, 0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, // Rising
    0x8000, 0x7000, 0x6000, 0x5000, 0x4000, 0x3000, 0x2000, 0x1000, // Falling
];

// Pattern 3: Sawtooth Wave - Sharp rise, quick fall
const SAWTOOTH_PATTERN: &[u16] = &[
    0x0000, 0x0800, 0x1000, 0x1800, 0x2000, 0x2800, 0x3000, 0x3800,
    0x4000, 0x4800, 0x5000, 0x5800, 0x6000, 0x6800, 0x7000, 0x7800,
    0x8000, 0x0000, 0x0800, 0x1000, 0x1800, 0x2000, 0x2800, 0x3000,
    0x3800, 0x4000, 0x4800, 0x5000, 0x5800, 0x6000, 0x6800, 0x7000,
    0x7800, 0x8000, 0x0000, 0x0800, 0x1000, 0x1800, 0x2000, 0x2800,
    0x3000, 0x3800, 0x4000, 0x4800, 0x5000, 0x5800, 0x6000, 0x6800,
    0x7000, 0x7800, 0x8000, 0x0000, 0x0800, 0x1000, 0x1800, 0x2000,
    0x2800, 0x3000, 0x3800, 0x4000, 0x4800, 0x5000, 0x5800, 0x6000,
];

// Pattern 4: Staircase - Digital steps creating a ladder effect
const STAIRCASE_PATTERN: &[u16] = &[
    0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, 0x1000, // Step 1
    0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, // Step 2
    0x3000, 0x3000, 0x3000, 0x3000, 0x3000, 0x3000, 0x3000, 0x3000, // Step 3
    0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, // Step 4
    0x5000, 0x5000, 0x5000, 0x5000, 0x5000, 0x5000, 0x5000, 0x5000, // Step 5
    0x6000, 0x6000, 0x6000, 0x6000, 0x6000, 0x6000, 0x6000, 0x6000, // Step 6
    0x7000, 0x7000, 0x7000, 0x7000, 0x7000, 0x7000, 0x7000, 0x7000, // Step 7
    0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, 0x8000, // Step 8
];

// Pattern 5: Heart Shape - Creates heart-like pattern on XY oscilloscope mode
const HEART_SHAPE_PATTERN: &[u16] = &[
    0x4000, 0x5000, 0x6000, 0x7000, 0x7800, 0x7000, 0x6000, 0x5000, // Right hump
    0x4000, 0x3000, 0x2000, 0x1000, 0x0800, 0x1000, 0x2000, 0x3000, // Left hump
    0x4000, 0x4800, 0x5000, 0x5800, 0x6000, 0x6800, 0x7000, 0x7800, // Peak
    0x7000, 0x6000, 0x5000, 0x4000, 0x3000, 0x2000, 0x1000, 0x0000, // Fall to point
    0x1000, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x7000, 0x7800, // Rise again
    0x7000, 0x6000, 0x5000, 0x4000, 0x3000, 0x2000, 0x1000, 0x0800, // Back down
    0x1000, 0x2000, 0x3000, 0x4000, 0x4800, 0x5000, 0x5800, 0x6000, // Gentle rise
    0x5800, 0x5000, 0x4800, 0x4000, 0x3800, 0x3000, 0x2800, 0x2000, // Return
];

// Pattern 6: House Shape - Simple house outline
const HOUSE_PATTERN: &[u16] = &[
    0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, // Foundation
    0x2000, 0x2800, 0x3000, 0x3800, 0x4000, 0x4800, 0x5000, 0x5800, // Left wall rising
    0x6000, 0x6800, 0x7000, 0x7800, 0x8000, 0x7800, 0x7000, 0x6800, // Roof peak
    0x6000, 0x5800, 0x5000, 0x4800, 0x4000, 0x3800, 0x3000, 0x2800, // Right wall down
    0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, // Foundation
    0x2000, 0x2000, 0x3000, 0x3000, 0x3000, 0x3000, 0x2000, 0x2000, // Door
    0x2000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x2000, // Window
    0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, 0x2000, // Base
];

// Pattern 7: Smiley Face - Creates a smiley when viewed properly
const SMILEY_FACE_PATTERN: &[u16] = &[
    // Circle outline (simplified)
    0x4000, 0x5000, 0x6000, 0x7000, 0x7800, 0x7000, 0x6000, 0x5000,
    0x4000, 0x3000, 0x2000, 0x1000, 0x0800, 0x1000, 0x2000, 0x3000,
    // Left eye
    0x3000, 0x3000, 0x3000, 0x3000, 0x4000, 0x4000, 0x4000, 0x4000,
    // Right eye  
    0x5000, 0x5000, 0x5000, 0x5000, 0x4000, 0x4000, 0x4000, 0x4000,
    // Smile (curved line)
    0x3000, 0x3200, 0x3400, 0x3800, 0x4000, 0x4800, 0x5400, 0x5200,
    0x5000, 0x4800, 0x4000, 0x3800, 0x3400, 0x3200, 0x3000, 0x4000,
    // Complete the circle
    0x4000, 0x5000, 0x6000, 0x7000, 0x7800, 0x7000, 0x6000, 0x5000,
    0x4000, 0x3000, 0x2000, 0x1000, 0x0800, 0x1000, 0x2000, 0x3000,
];

const SAMPLE_RATE: u32 = 8000; // Lower sample rate for cleaner oscilloscope viewing
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes
const TX_BUFFER_SIZE: usize = 256;
const RX_BUFFER_SIZE: usize = 256;

#[entry]
fn main() -> ! {
    println!("🎨 ESP32-H2 I2S VISUAL PATTERN GENERATOR 🎨");
    println!("📺 Creates oscilloscope-friendly visual patterns!");
    
    let peripherals = Peripherals::take();
    println!("✅ Peripherals acquired");
    
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    println!("✅ System and clocks initialized");
    
    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    println!("✅ GPIO initialized");

    // Status LED
    let mut led = Output::new(io.pins.gpio8, Level::Low);
    println!("✅ Status LED on GPIO8 configured");

    // LED startup sequence
    for i in 1..=5 {
        println!("💡 Startup blink {}/5", i);
        led.set_high();
        delay.delay_millis(150);
        led.set_low();
        delay.delay_millis(150);
    }

    // Initialize DMA
    println!("🔧 Initializing DMA...");
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0.configure(false, DmaPriority::Priority0);
    println!("✅ DMA configured");

    println!("📊 Creating DMA buffers ({} bytes each)...", TX_BUFFER_SIZE);
    let (_rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = 
        dma_buffers!(RX_BUFFER_SIZE, TX_BUFFER_SIZE);
    println!("✅ DMA buffers created");

    // Create I2S instance
    println!("🎵 Creating I2S instance...");
    let i2s = I2s::new(
        peripherals.I2S0,
        I2S_STANDARD,
        I2S_DATA_FORMAT,
        SAMPLE_RATE.Hz(),
        dma_channel,
        rx_descriptors,
        tx_descriptors,
        &clocks,
    );
    println!("✅ I2S instance created");

    // Configure I2S TX pins
    println!("📌 Configuring I2S TX pins...");
    let bclk = io.pins.gpio4;
    let ws = io.pins.gpio5;
    let dout = io.pins.gpio6;
    
    println!("🔧 Building I2S TX interface...");
    let mut i2s_tx = i2s.i2s_tx
        .with_bclk(bclk)
        .with_ws(ws)
        .with_dout(dout)
        .build();
    
    println!("✅ I2S TX Configuration Complete:");
    println!("   🔌 BCLK: GPIO4 (Bit Clock)");
    println!("   🔌 WS:   GPIO5 (Word Select/Frame Sync)"); 
    println!("   🔌 DOUT: GPIO6 (Data Out)");
    println!("   📊 Sample Rate: {} Hz", SAMPLE_RATE);
    println!("   🎼 Format: 16-bit, Philips I2S");
    println!();
    println!("🎨 OSCILLOSCOPE VIEWING GUIDE:");
    println!("   📺 Connect oscilloscope to GPIO6 (DOUT)");
    println!("   ⚡ Set trigger to rising edge");
    println!("   📏 Time scale: ~1ms/div for best viewing");
    println!("   📈 Voltage scale: ~1V/div");
    println!("   🎯 Look for geometric patterns!");
    println!();
    println!("🚀 Pattern sequence:");
    println!("   1️⃣  Square Waves - Clean rectangles");
    println!("   2️⃣  Triangle Waves - Smooth ramps");
    println!("   3️⃣  Sawtooth Waves - Sharp rises");
    println!("   4️⃣  Staircase - Digital steps");
    println!("   5️⃣  Heart Shape - Romantic curves");
    println!("   6️⃣  House Pattern - Architectural lines");
    println!("   7️⃣  Smiley Face - Happy curves");
    println!();

    let mut pattern_index = 0;
    let mut transmission_count = 0;

    loop {
        transmission_count += 1;
        led.set_high();
        
        let current_pattern = VISUAL_PATTERNS[pattern_index];
        let pattern_names = [
            "Square Wave", "Triangle Wave", "Sawtooth", "Staircase", 
            "Heart Shape", "House Pattern", "Smiley Face"
        ];
        
        println!("🎨 === PATTERN {}: {} === (Transmission #{}) 🎨", 
                 pattern_index + 1, pattern_names[pattern_index], transmission_count);
        
        // Send pattern multiple times for good oscilloscope capture
        for repeat in 0..10 {
            println!("   📡 Repeat {}/10 - Transmitting {} samples", repeat + 1, current_pattern.len());
            
            // Send the pattern
            match i2s_tx.write(current_pattern) {
                Ok(_) => {
                    if repeat % 3 == 0 {
                        println!("   ✅ Pattern sent successfully");
                    }
                }
                Err(e) => {
                    println!("   ❌ Error sending pattern: {:?}", e);
                    break;
                }
            }
            
            delay.delay_millis(10); // Small delay between repeats
        }
        
        led.set_low();
        
        println!("✅ Pattern {} complete!", pattern_names[pattern_index]);
        println!("   📊 Pattern size: {} samples", current_pattern.len());
        println!("   🎯 Check oscilloscope for visual pattern!");
        
        // Move to next pattern
        pattern_index = (pattern_index + 1) % VISUAL_PATTERNS.len();
        
        if pattern_index == 0 {
            println!("\n🎉 Completed full pattern cycle! Starting over...\n");
        }
        
        println!("⏳ Waiting 2 seconds before next pattern...\n");
        delay.delay_millis(2000);
    }
}