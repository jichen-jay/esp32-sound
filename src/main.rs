//! ESP32 I2S Audio Communication System
//! ESP32-C6 (Sender) ↔ ESP32-H2 (Receiver)
//! Transmits Happy Birthday song between two ESP32 devices

#![no_std]
#![no_main]

use esp_hal::{
    delay::Delay,
    dma_buffers,
    gpio::{Io, Level, Output},
    i2s::master::{DataFormat, I2s, Standard},  // Updated import
    prelude::*,
    time::Rate,
};
use esp_println::println;
use core::panic::PanicInfo;
use esp_backtrace as _;

// Mock audio data for testing - replace with your generated audio
const HAPPY_BIRTHDAY_AUDIO: &[u8] = &[
    0x00, 0x00, 0x00, 0x10, 0x00, 0x20, 0x00, 0x30,
    0x00, 0x40, 0x00, 0x50, 0x00, 0x60, 0x00, 0x70,
    0x00, 0x80, 0x00, 0x90, 0x00, 0xA0, 0x00, 0xB0,
    0x00, 0xC0, 0x00, 0xD0, 0x00, 0xE0, 0x00, 0xF0,
    // Add more data or include your generated happy_birthday_audio.rs
];

const SAMPLE_RATE: u32 = 16000;

// I2S Configuration
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes
const TX_BUFFER_SIZE: usize = 1024;
const RX_BUFFER_SIZE: usize = 1024;

// Device mode - change this based on which device you're flashing
// ESP32-C6 = Sender, ESP32-H2 = Receiver
#[cfg(feature = "esp32c6")]
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

#[cfg(feature = "esp32h2")]
const DEVICE_MODE: DeviceMode = DeviceMode::Receiver;

// Default to sender if no feature is specified
#[cfg(not(any(feature = "esp32c6", feature = "esp32h2")))]
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

#[derive(Clone, Copy, Debug)]
enum DeviceMode {
    Sender,
    Receiver,
}

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let delay = Delay::new();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Updated DMA channel access
    #[cfg(any(esp32, esp32s2))]
    let dma_channel = peripherals.DMA_I2S0;
    
    #[cfg(not(any(esp32, esp32s2)))]
    let dma_channel = peripherals.DMA_CH0;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = 
        dma_buffers!(RX_BUFFER_SIZE, TX_BUFFER_SIZE);

    // Updated I2S construction
    let i2s = I2s::new(
        peripherals.I2S0,
        I2S_STANDARD,
        I2S_DATA_FORMAT,
        Rate::from_hz(SAMPLE_RATE),
        dma_channel,
    );

    match DEVICE_MODE {
        DeviceMode::Sender => {
            let i2s_tx = i2s
                .i2s_tx
                .with_bclk(io.pins.gpio4)
                .with_ws(io.pins.gpio5)
                .with_dout(io.pins.gpio6)
                .build(tx_descriptors);
            
            run_sender(i2s_tx, tx_buffer, io, delay);
        }
        DeviceMode::Receiver => {
            let i2s_rx = i2s
                .i2s_rx
                .with_bclk(io.pins.gpio4)
                .with_ws(io.pins.gpio5)
                .with_din(io.pins.gpio6)
                .build(rx_descriptors);
            
            run_receiver(i2s_rx, rx_buffer, io, delay);
        }
    }
}

fn run_sender(
    mut i2s_tx: esp_hal::i2s::master::I2sTx<'static, esp_hal::Blocking>,  // Correct type
    tx_buffer: &'static mut [u8],
    io: Io,
    mut delay: Delay,
) -> ! {
    println!("Initializing ESP32-C6 as SENDER");

    // Status LED - using GPIO8 (commonly available on dev boards)
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    println!("I2S TX configured:");
    println!("  BCLK: GPIO4");
    println!("  WS:   GPIO5"); 
    println!("  DOUT: GPIO6");
    println!("Audio data size: {} bytes", HAPPY_BIRTHDAY_AUDIO.len());

    let mut transmission_count = 0;

    loop {
        transmission_count += 1;
        led.set_high();
        
        println!("=== Transmission #{} ===", transmission_count);
        
        // Copy audio data to DMA buffer in chunks
        let mut offset = 0;
        while offset < HAPPY_BIRTHDAY_AUDIO.len() {
            let chunk_size = (HAPPY_BIRTHDAY_AUDIO.len() - offset).min(tx_buffer.len());
            
            // Copy audio chunk to DMA buffer
            tx_buffer[..chunk_size].copy_from_slice(
                &HAPPY_BIRTHDAY_AUDIO[offset..offset + chunk_size]
            );
            
            // Convert bytes to u16 for I2S transmission
            let words = unsafe { 
                core::slice::from_raw_parts(
                    tx_buffer.as_ptr() as *const u16,
                    chunk_size / 2
                )
            };
            
            println!("TX: Sending {} bytes (offset: {})", chunk_size, offset);
            
            match i2s_tx.write_words(words) {
                Ok(_) => {
                    println!("TX: Chunk sent successfully");
                }
                Err(e) => {
                    println!("TX: I2S write error: {:?}", e);
                }
            }
            
            offset += chunk_size;
            delay.delay_millis(50); // Delay between chunks
        }
        
        led.set_low();
        println!("TX: Transmission complete, waiting 3 seconds...");
        delay.delay_millis(3000);
    }
}

fn run_receiver(
    mut i2s_rx: esp_hal::i2s::master::I2sRx<'static, esp_hal::Blocking>,  // Correct type
    rx_buffer: &'static mut [u8],
    io: Io,
    mut delay: Delay,
) -> ! {
    println!("Initializing ESP32-H2 as RECEIVER");

    // Status LED - using GPIO8 
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    println!("I2S RX configured:");
    println!("  BCLK: GPIO4");
    println!("  WS:   GPIO5");
    println!("  DIN:  GPIO6");
    println!("Waiting for audio data...");

    let mut reception_count = 0;

    loop {
        led.set_high();
        
        // Convert buffer to u16 for I2S reception
        let words = unsafe {
            core::slice::from_raw_parts_mut(
                rx_buffer.as_mut_ptr() as *mut u16,
                rx_buffer.len() / 2
            )
        };
        
        // Receive audio data
        match i2s_rx.read_words(words) {
            Ok(_) => {
                reception_count += 1;
                println!("RX: Received {} bytes (reception #{})", rx_buffer.len(), reception_count);
                process_received_audio(rx_buffer);
            }
            Err(e) => {
                println!("RX: I2S read error: {:?}", e);
            }
        }
        
        led.set_low();
        delay.delay_millis(100);
    }
}

fn process_received_audio(audio_data: &[u8]) {
    // Basic audio processing - calculate average amplitude
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
        if abs_sample > 0 {
            non_zero_samples += 1;
        }
    }
    
    let avg_amplitude = if samples.len() > 0 { 
        sum / samples.len() as i32 
    } else { 
        0 
    };
    
    println!("Audio Analysis:");
    println!("  Samples: {}, Non-zero: {}", samples.len(), non_zero_samples);
    println!("  Avg amplitude: {}, Max: {}", avg_amplitude, max_amplitude);
    
    // Detect signal strength
    if avg_amplitude > 1000 {
        println!("  🔊 Strong audio signal detected!");
    } else if avg_amplitude > 100 {
        println!("  🔉 Weak audio signal detected");
    } else if non_zero_samples > 0 {
        println!("  📶 Minimal signal detected");
    } else {
        println!("  🔇 No audio signal");
    }
    
    // Show first few samples for debugging
    if samples.len() >= 4 {
        println!("  First samples: {} {} {} {}", 
                samples[0], samples[1], samples[2], samples[3]);
    }
}
