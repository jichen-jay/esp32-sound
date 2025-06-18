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
    i2s::{DataFormat, I2s, I2sWrite, I2sRead, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;
use esp_backtrace as _;

// Include the generated Happy Birthday audio data
// You'll need to generate this with your Python script
include!("./happy_birthday_audio.rs");

// I2S Configuration
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes
const TX_BUFFER_SIZE: usize = 1024;
const RX_BUFFER_SIZE: usize = 1024;

// Device mode - change this based on which device you're flashing
#[cfg(feature = "esp32c6")]
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

#[cfg(feature = "esp32h2")]
const DEVICE_MODE: DeviceMode = DeviceMode::Receiver;

#[derive(Clone, Copy, Debug)]
enum DeviceMode {
    Sender,
    Receiver,
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    
    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Initialize DMA
    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0.configure(false, DmaPriority::Priority0);

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = 
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
                    
                    // Send the chunk using blocking write
                    i2s_tx.write(&tx_buffer[..chunk_size]).unwrap();
                    
                    println!("  Chunk {}: {} bytes sent", chunk_count, chunk_size);
                    
                    bytes_sent += chunk_size;
                }
                
                led.set_low();
                println!("Transmission complete! Sent {} bytes in {} chunks", bytes_sent, chunk_count);
                println!("Waiting 3 seconds before next transmission...\n");
                delay.delay_millis(3000);
            }
        }
        DeviceMode::Receiver => {
            println!("Configuring as RECEIVER (ESP32-H2)");
            
            // ESP32-H2 pins for I2S RX - using same GPIO numbers for convenience
            let bclk = io.pins.gpio4;   // Same as sender
            let ws = io.pins.gpio5;     // Same as sender
            let din = io.pins.gpio6;    // Same pin as sender's DOUT
            
            let mut i2s_rx = i2s.i2s_rx
                .with_bclk(bclk)
                .with_ws(ws)
                .with_din(din)
                .build();
            
            // Status LED - use available GPIO on H2
            let mut led = Output::new(io.pins.gpio8, Level::Low);
            
            // Receiver loop
            println!("=== ESP32-H2 RECEIVER INITIALIZED ===");
            println!("I2S RX Configuration:");
            println!("  BCLK: GPIO4");
            println!("  WS:   GPIO5");
            println!("  DIN:  GPIO6");
            println!("  Sample Rate: {} Hz", SAMPLE_RATE);
            println!("Listening for audio data...\n");

            let mut reception_count = 0;
            let mut total_bytes_received = 0;

            loop {
                led.set_high();
                
                // Receive audio data using blocking read
                i2s_rx.read(rx_buffer).unwrap();
                
                reception_count += 1;
                total_bytes_received += rx_buffer.len();
                
                println!("Reception #{}: {} bytes", reception_count, rx_buffer.len());
                
                // Process the received audio
                process_received_audio(rx_buffer);
                
                led.set_low();
                delay.delay_millis(10);
            }
        }
    }
}

fn process_received_audio(audio_data: &[u8]) {
    // Convert bytes to i16 samples for analysis
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