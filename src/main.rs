//! ESP32 I2S Audio Communication System (Bare Metal)
//! ESP32-C6 (Sender) ↔ ESP32-H2 (Receiver)
//! Transmits Happy Birthday song between two ESP32 devices

#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_buffers,
    gpio::{Io, Level, Output},
    i2s::{DataFormat, I2s, I2sRead, I2sWrite, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;
use esp_backtrace as _;

// Happy Birthday audio data - 16kHz, 16-bit, mono
// This is a simplified version - you can replace with your generated data
const HAPPY_BIRTHDAY_AUDIO: &[u8] = &[
    // Simple sine wave pattern for demonstration
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
];

const SAMPLE_RATE: u32 = 16000;

// I2S Configuration
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes
const TX_BUFFER_SIZE: usize = 512;
const RX_BUFFER_SIZE: usize = 512;

// Device mode - ESP32-C6 is sender by default
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

#[derive(Clone, Copy, Debug)]
enum DeviceMode {
    Sender,
    #[allow(dead_code)]
    Receiver,
}

#[entry]
fn main() -> ! {

       esp_println::println!("=== APPLICATION STARTING ===");
    
    let peripherals = Peripherals::take();
    esp_println::println!("Peripherals taken");
    
    let system = SystemControl::new(peripherals.SYSTEM);
    esp_println::println!("System control initialized");
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    
    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Initialize DMA
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0.configure(false, DmaPriority::Priority0);

    let (_rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = 
        dma_buffers!(RX_BUFFER_SIZE, TX_BUFFER_SIZE);

    // Create I2S instance
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

    match DEVICE_MODE {
        DeviceMode::Sender => {
            println!("Configuring as SENDER (ESP32-C6)");
            
            // ESP32-C6 pins for I2S TX
            let bclk = io.pins.gpio4;
            let ws = io.pins.gpio5;
            let dout = io.pins.gpio6;
            
            let mut i2s_tx = i2s.i2s_tx
                .with_bclk(bclk)
                .with_ws(ws)
                .with_dout(dout)
                .build();
            
            // Status LED
            let mut led = Output::new(io.pins.gpio8, Level::Low);
            
            // Sender loop
            println!("=== ESP32-C6 SENDER INITIALIZED ===");
            println!("I2S TX Configuration:");
            println!("  BCLK: GPIO4");
            println!("  WS:   GPIO5"); 
            println!("  DOUT: GPIO6");
            println!("  Sample Rate: {} Hz", SAMPLE_RATE);
            println!("  Audio Data Size: {} bytes", HAPPY_BIRTHDAY_AUDIO.len());

            let mut transmission_count = 0;

            loop {
                transmission_count += 1;
                led.set_high();
                
                println!("\n=== Transmission #{} ===", transmission_count);
                
                // Send audio data in chunks
                let mut bytes_sent = 0;
                let mut chunk_count = 0;
                
                while bytes_sent < HAPPY_BIRTHDAY_AUDIO.len() {
                    let remaining = HAPPY_BIRTHDAY_AUDIO.len() - bytes_sent;
                    let chunk_size = remaining.min(tx_buffer.len());
                    
                    // Copy audio chunk to DMA buffer
                    tx_buffer[..chunk_size].copy_from_slice(
                        &HAPPY_BIRTHDAY_AUDIO[bytes_sent..bytes_sent + chunk_size]
                    );
                    
                    chunk_count += 1;
                    
                    // Convert bytes to u16 words for I2S
                    let mut words: [u16; 256] = [0; 256]; // Max chunk size / 2
                    let word_count = chunk_size / 2;
                    
                    for i in 0..word_count {
                        if i * 2 + 1 < chunk_size {
                            // Convert little-endian bytes to u16
                            words[i] = u16::from_le_bytes([
                                tx_buffer[i * 2],
                                tx_buffer[i * 2 + 1]
                            ]);
                        }
                    }
                    
                    // Send the chunk using blocking write
                    match i2s_tx.write(&words[..word_count]) {
                        Ok(_) => {
                            println!("  Chunk {}: {} words sent", chunk_count, word_count);
                        }
                        Err(e) => {
                            println!("  Error sending chunk {}: {:?}", chunk_count, e);
                            break;
                        }
                    }
                    
                    bytes_sent += chunk_size;
                    delay.delay_millis(10); // Small delay between chunks
                }
                
                led.set_low();
                println!("Transmission complete! Sent {} bytes in {} chunks", bytes_sent, chunk_count);
                println!("Waiting 3 seconds before next transmission...\n");
                delay.delay_millis(3000);
            }
        }
        DeviceMode::Receiver => {
            println!("Configuring as RECEIVER");
            
            // ESP32 pins for I2S RX
            let bclk = io.pins.gpio4;
            let ws = io.pins.gpio5;
            let din = io.pins.gpio6;
            
            let mut i2s_rx = i2s.i2s_rx
                .with_bclk(bclk)
                .with_ws(ws)
                .with_din(din)
                .build();
            
            // Status LED
            let mut led = Output::new(io.pins.gpio8, Level::Low);
            
            // Receiver loop
            println!("=== ESP32 RECEIVER INITIALIZED ===");
            println!("I2S RX Configuration:");
            println!("  BCLK: GPIO4");
            println!("  WS:   GPIO5");
            println!("  DIN:  GPIO6");
            println!("  Sample Rate: {} Hz", SAMPLE_RATE);
            println!("Listening for audio data...\n");

            let mut reception_count = 0;

            loop {
                led.set_high();
                
                // Prepare buffer for receiving u16 words
                let mut word_buffer: [u16; 256] = [0; 256];
                
                // Receive audio data using blocking read
                match i2s_rx.read(&mut word_buffer) {
                    Ok(_) => {
                        reception_count += 1;
                        println!("Reception #{}: {} words", reception_count, word_buffer.len());
                        
                        // Convert u16 words back to bytes for processing
                        let mut byte_buffer: [u8; 512] = [0; 512];
                        for (i, &word) in word_buffer.iter().enumerate() {
                            let bytes = word.to_le_bytes();
                            if i * 2 + 1 < byte_buffer.len() {
                                byte_buffer[i * 2] = bytes[0];
                                byte_buffer[i * 2 + 1] = bytes[1];
                            }
                        }
                        
                        // Process the received audio
                        process_received_audio(&byte_buffer[..word_buffer.len() * 2]);
                    }
                    Err(e) => {
                        println!("Error receiving audio: {:?}", e);
                    }
                }
                
                led.set_low();
                delay.delay_millis(10);
            }
        }
    }
}

fn process_received_audio(audio_data: &[u8]) {
    // Convert bytes to i16 samples for analysis
    if audio_data.len() < 2 {
        println!("   🔇 No data");
        return;
    }
    
    let samples = unsafe {
        core::slice::from_raw_parts(
            audio_data.as_ptr() as *const i16,
            audio_data.len() / 2
        )
    };
    
    let mut sum: i32 = 0;
    let mut max_amplitude: i16 = 0;
    let mut non_zero_samples = 0;
    
    for &sample in samples.iter() {
        let abs_sample = sample.abs();
        sum += abs_sample as i32;
        max_amplitude = max_amplitude.max(abs_sample);
        if abs_sample > 10 { // Threshold to ignore noise
            non_zero_samples += 1;
        }
    }
    
    let avg_amplitude = if samples.len() > 0 { 
        sum / samples.len() as i32 
    } else { 
        0 
    };
    
    println!("📊 Audio Analysis:");
    println!("   Samples: {} | Active: {} ({:.1}%)", 
             samples.len(),
             non_zero_samples, 
             (non_zero_samples as f32 / samples.len() as f32) * 100.0);
    println!("   Avg: {} | Peak: {}", avg_amplitude, max_amplitude);
    
    // Signal strength detection
    if avg_amplitude > 1000 {
        println!("   🔊 Strong audio signal!");
    } else if avg_amplitude > 100 {
        println!("   🔉 Moderate signal");
    } else if non_zero_samples > 10 {
        println!("   📶 Weak signal");
    } else {
        println!("   🔇 No signal");
    }
}