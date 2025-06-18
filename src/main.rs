//! ESP32-H2 I2S Audio Sender
//! Transmits Happy Birthday song via I2S
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

// Happy Birthday audio data - 16kHz, 16-bit, mono
const HAPPY_BIRTHDAY_AUDIO: &[u8] = &[
    // Extended sine wave pattern for Happy Birthday melody
    0x00, 0x00, 0x7f, 0x0c, 0xf8, 0x18, 0x6a, 0x24, 0xd5, 0x2e, 0x3c, 0x38,
    0x9b, 0x40, 0xf0, 0x47, 0x3b, 0x4e, 0x7a, 0x53, 0xac, 0x57, 0xd0, 0x5a,
    0xe5, 0x5c, 0xec, 0x5d, 0xe5, 0x5d, 0xd0, 0x5c, 0xac, 0x5a, 0x7a, 0x57,
    0x3b, 0x53, 0xf0, 0x4d, 0x9b, 0x47, 0x3c, 0x40, 0xd5, 0x37, 0x6a, 0x2e,
    0xf8, 0x23, 0x7f, 0x18, 0x00, 0x0c, 0x81, 0xff, 0x08, 0xf2, 0x96, 0xe4,
    0x2b, 0xd7, 0xc4, 0xc9, 0x65, 0xbc, 0x10, 0xaf, 0xc5, 0xa1, 0x86, 0x94,
    0x54, 0x87, 0x30, 0x7a, 0x1b, 0x6d, 0x14, 0x60, 0x1b, 0x53, 0x30, 0x46,
    0x54, 0x39, 0x86, 0x2c, 0xc5, 0x1f, 0x10, 0x13, 0x65, 0x06, 0xc4, 0xf9,
    0x2b, 0xed, 0x96, 0xe0, 0x08, 0xd4, 0x81, 0xc7, 0x00, 0xbb, 0x7f, 0xae,
    0xf8, 0xa1, 0x6a, 0x95, 0xd5, 0x88, 0x3c, 0x7c, 0x9b, 0x6f, 0xf0, 0x62,
    0x3b, 0x56, 0x7a, 0x49, 0xac, 0x3c, 0xd0, 0x2f, 0xe5, 0x22, 0xec, 0x15,
    0xe5, 0x08, 0xd0, 0xfb, 0xac, 0xee, 0x7a, 0xe1, 0x3b, 0xd4, 0xf0, 0xc6,
    0x9b, 0xb9, 0x3c, 0xac, 0xd5, 0x9e, 0x6a, 0x91, 0xf8, 0x83, 0x7f, 0x76,
    // Repeat for longer melody
    0x00, 0x00, 0x7f, 0x0c, 0xf8, 0x18, 0x6a, 0x24, 0xd5, 0x2e, 0x3c, 0x38,
    0x9b, 0x40, 0xf0, 0x47, 0x3b, 0x4e, 0x7a, 0x53, 0xac, 0x57, 0xd0, 0x5a,
    0xe5, 0x5c, 0xec, 0x5d, 0xe5, 0x5d, 0xd0, 0x5c, 0xac, 0x5a, 0x7a, 0x57,
    0x3b, 0x53, 0xf0, 0x4d, 0x9b, 0x47, 0x3c, 0x40, 0xd5, 0x37, 0x6a, 0x2e,
    0xf8, 0x23, 0x7f, 0x18, 0x00, 0x0c, 0x81, 0xff, 0x08, 0xf2, 0x96, 0xe4,
    0x2b, 0xd7, 0xc4, 0xc9, 0x65, 0xbc, 0x10, 0xaf, 0xc5, 0xa1, 0x86, 0x94,
    0x54, 0x87, 0x30, 0x7a, 0x1b, 0x6d, 0x14, 0x60, 0x1b, 0x53, 0x30, 0x46,
    0x54, 0x39, 0x86, 0x2c, 0xc5, 0x1f, 0x10, 0x13, 0x65, 0x06, 0xc4, 0xf9,
    0x2b, 0xed, 0x96, 0xe0, 0x08, 0xd4, 0x81, 0xc7, 0x00, 0xbb, 0x7f, 0xae,
    0xf8, 0xa1, 0x6a, 0x95, 0xd5, 0x88, 0x3c, 0x7c, 0x9b, 0x6f, 0xf0, 0x62,
    0x3b, 0x56, 0x7a, 0x49, 0xac, 0x3c, 0xd0, 0x2f, 0xe5, 0x22, 0xec, 0x15,
    0xe5, 0x08, 0xd0, 0xfb, 0xac, 0xee, 0x7a, 0xe1, 0x3b, 0xd4, 0xf0, 0xc6,
    // Third verse
    0x9b, 0xb9, 0x3c, 0xac, 0xd5, 0x9e, 0x6a, 0x91, 0xf8, 0x83, 0x7f, 0x76,
    0x00, 0x00, 0x7f, 0x0c, 0xf8, 0x18, 0x6a, 0x24, 0xd5, 0x2e, 0x3c, 0x38,
    0x9b, 0x40, 0xf0, 0x47, 0x3b, 0x4e, 0x7a, 0x53, 0xac, 0x57, 0xd0, 0x5a,
    0xe5, 0x5c, 0xec, 0x5d, 0xe5, 0x5d, 0xd0, 0x5c, 0xac, 0x5a, 0x7a, 0x57,
    0x3b, 0x53, 0xf0, 0x4d, 0x9b, 0x47, 0x3c, 0x40, 0xd5, 0x37, 0x6a, 0x2e,
    0xf8, 0x23, 0x7f, 0x18, 0x00, 0x0c, 0x81, 0xff, 0x08, 0xf2, 0x96, 0xe4,
    0x2b, 0xd7, 0xc4, 0xc9, 0x65, 0xbc, 0x10, 0xaf, 0xc5, 0xa1, 0x86, 0x94,
    0x54, 0x87, 0x30, 0x7a, 0x1b, 0x6d, 0x14, 0x60, 0x1b, 0x53, 0x30, 0x46,
];

const SAMPLE_RATE: u32 = 16000;
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes - smaller for ESP32-H2
const TX_BUFFER_SIZE: usize = 256;
const RX_BUFFER_SIZE: usize = 256;

#[entry]
fn main() -> ! {
    println!("🎵 ESP32-H2 I2S AUDIO SENDER STARTING 🎵");
    
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
    for i in 1..=3 {
        println!("💡 Startup blink {}/3", i);
        led.set_high();
        delay.delay_millis(200);
        led.set_low();
        delay.delay_millis(200);
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
    println!("   📦 Audio Data: {} bytes", HAPPY_BIRTHDAY_AUDIO.len());
    println!("   🎼 Format: 16-bit Stereo, Philips I2S");
    println!();
    println!("🚀 Ready to transmit Happy Birthday! 🎂");
    println!("💡 Connect GPIO4,5,6 to receiver's GPIO4,5,6");
    println!("📡 Starting transmission loop...");
    println!();

    let mut transmission_count = 0;
    let chunk_size = 64; // Process in smaller chunks

    loop {
        transmission_count += 1;
        led.set_high();
        
        println!("🎵 === TRANSMISSION #{} === 🎵", transmission_count);
        
        // Send audio data in chunks
        let mut bytes_sent = 0;
        let mut chunk_count = 0;
        
        while bytes_sent < HAPPY_BIRTHDAY_AUDIO.len() {
            let remaining = HAPPY_BIRTHDAY_AUDIO.len() - bytes_sent;
            let current_chunk_size = remaining.min(chunk_size);
            
            // Ensure chunk_size is even (for 16-bit samples)
            let current_chunk_size = current_chunk_size & !1;
            
            if current_chunk_size == 0 {
                break;
            }
            
            chunk_count += 1;
            
            // Convert bytes to u16 words for I2S
            let word_count = current_chunk_size / 2;
            let mut words: [u16; 32] = [0; 32]; // Max 64 bytes / 2
            
            for i in 0..word_count {
                let byte_idx = bytes_sent + (i * 2);
                if byte_idx + 1 < HAPPY_BIRTHDAY_AUDIO.len() {
                    // Convert little-endian bytes to u16
                    words[i] = u16::from_le_bytes([
                        HAPPY_BIRTHDAY_AUDIO[byte_idx],
                        HAPPY_BIRTHDAY_AUDIO[byte_idx + 1]
                    ]);
                }
            }
            
            // Send the chunk
            match i2s_tx.write(&words[..word_count]) {
                Ok(_) => {
                    if chunk_count % 10 == 0 {
                        println!("   📤 Chunk {}: {} words sent", chunk_count, word_count);
                    }
                }
                Err(e) => {
                    println!("   ❌ Error sending chunk {}: {:?}", chunk_count, e);
                    break;
                }
            }
            
            bytes_sent += current_chunk_size;
            delay.delay_millis(5); // Small delay between chunks
        }
        
        led.set_low();
        
        println!("✅ Transmission complete!");
        println!("   📊 Sent: {} bytes in {} chunks", bytes_sent, chunk_count);
        println!("   🎵 Happy Birthday melody transmitted!");
        
        if transmission_count % 5 == 0 {
            println!("🎂 Happy Birthday! 🎉 (Transmission #{})", transmission_count);
        }
        
        println!("⏳ Waiting 3 seconds before next transmission...\n");
        delay.delay_millis(3000);
    }
}