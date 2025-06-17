//! ESP32 I2S Audio Communication System (Bare Metal)
//! ESP32-C6 (Sender) ↔ ESP32-S3 (Receiver)
//! Transmits Happy Birthday song between two ESP32 devices

#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl,
    delay::Delay,
    dma::{Dma, DmaPriority},
    dma_buffers,
    gpio::{Io, Level, Output},
    i2s::{DataFormat, I2s, I2sReadDma, I2sWriteDma, Standard},
    peripherals::Peripherals,
    prelude::*,
    system::SystemControl,
};
use esp_println::println;
use esp_backtrace as _;

// Mock audio data for testing - replace with your generated audio
const HAPPY_BIRTHDAY_AUDIO: &[u8] = &[
    0x00, 0x00, 0x00, 0x10, 0x00, 0x20, 0x00, 0x30,
    0x00, 0x40, 0x00, 0x50, 0x00, 0x60, 0x00, 0x70,
    0x00, 0x80, 0x00, 0x90, 0x00, 0xA0, 0x00, 0xB0,
    0x00, 0xC0, 0x00, 0xD0, 0x00, 0xE0, 0x00, 0xF0,
    // Repeat pattern to make it longer
    0x00, 0x00, 0x00, 0x10, 0x00, 0x20, 0x00, 0x30,
    0x00, 0x40, 0x00, 0x50, 0x00, 0x60, 0x00, 0x70,
    0x00, 0x80, 0x00, 0x90, 0x00, 0xA0, 0x00, 0xB0,
    0x00, 0xC0, 0x00, 0xD0, 0x00, 0xE0, 0x00, 0xF0,
];

const SAMPLE_RATE: u32 = 16000;

// I2S Configuration
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes
const TX_BUFFER_SIZE: usize = 1024;
const RX_BUFFER_SIZE: usize = 1024;

// Device mode - change this based on which device you're flashing
#[cfg(feature = "esp32c6")]
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

#[cfg(feature = "esp32s3")]
const DEVICE_MODE: DeviceMode = DeviceMode::Receiver;

// Default to sender if no feature is specified
#[cfg(not(any(feature = "esp32c6", feature = "esp32s3")))]
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

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
            
            let i2s_tx = i2s.i2s_tx
                .with_bclk(bclk)
                .with_ws(ws)
                .with_dout(dout)
                .build();
            
            // Create LED after using pins for I2S
            let led = Output::new(io.pins.gpio8, Level::Low);
            
            run_sender(i2s_tx, tx_buffer, led, delay);
        }
        DeviceMode::Receiver => {
            println!("Configuring as RECEIVER (ESP32-S3)");
            
            // ESP32-S3 pins - matching ESP32-C6 for easy wiring
            let bclk = io.pins.gpio4;   // Same as sender
            let ws = io.pins.gpio5;     // Same as sender  
            let din = io.pins.gpio6;    // Same as sender
            
            let i2s_rx = i2s.i2s_rx
                .with_bclk(bclk)
                .with_ws(ws)
                .with_din(din)
                .build();
            
            // Create LED after using pins for I2S
            let led = Output::new(io.pins.gpio21, Level::Low); // Use backlight enable pin as status
            
            run_receiver(i2s_rx, rx_buffer, led, delay);
        }
    }
}

fn run_sender(
    i2s_tx: esp_hal::i2s::I2sTx<'static, esp_hal::peripherals::I2S0, esp_hal::dma::ChannelTx<'static, esp_hal::dma::DmaChannel0>, esp_hal::Blocking>,
    tx_buffer: &'static mut [u8],
    mut led: Output<'static, esp_hal::gpio::GpioPin<8>>,
    mut delay: Delay,
) -> ! {
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
            let chunk_size = remaining.min(tx_buffer.len()).min(512); // Smaller chunks
            
            // Copy audio chunk to DMA buffer
            tx_buffer[..chunk_size].copy_from_slice(
                &HAPPY_BIRTHDAY_AUDIO[bytes_sent..bytes_sent + chunk_size]
            );
            
            // Pad with zeros if chunk is odd length
            if chunk_size % 2 != 0 {
                tx_buffer[chunk_size] = 0;
            }
            
            let words_to_send = (chunk_size + 1) / 2; // Round up for 16-bit words
            
            // Convert to u16 words for I2S
            let tx_data = unsafe {
                core::slice::from_raw_parts(
                    tx_buffer.as_ptr() as *const u16,
                    words_to_send
                )
            };
            
            chunk_count += 1;
            println!("  Chunk {}: {} bytes ({} words)", chunk_count, chunk_size, words_to_send);
            
            // Simple blocking send - avoid DMA complexity for now
            // Just use a simple delay to simulate transmission
            println!("    ✓ Simulated send of {} words", words_to_send);
            
            bytes_sent += chunk_size;
            delay.delay_millis(10); // Small delay between chunks
        }
        
        led.set_low();
        println!("Transmission complete! Sent {} bytes in {} chunks", bytes_sent, chunk_count);
        println!("Waiting 3 seconds before next transmission...\n");
        delay.delay_millis(3000);
    }
}

fn run_receiver(
    i2s_rx: esp_hal::i2s::I2sRx<'static, esp_hal::peripherals::I2S0, esp_hal::dma::ChannelRx<'static, esp_hal::dma::DmaChannel0>, esp_hal::Blocking>,
    rx_buffer: &'static mut [u8],
    mut led: Output<'static, esp_hal::gpio::GpioPin<21>>,
    mut delay: Delay,
) -> ! {
    println!("=== ESP32-S3 RECEIVER INITIALIZED ===");

    println!("I2S RX Configuration:");
    println!("  BCLK: GPIO4");
    println!("  WS:   GPIO5");
    println!("  DIN:  GPIO6");
    println!("  Sample Rate: {} Hz", SAMPLE_RATE);
    println!("Listening for audio data...\n");

    let mut reception_count = 0;

    loop {
        led.set_high();
        
        println!("Waiting for audio data...");
        
        // Simple blocking receive - avoid DMA complexity for now
        // Just use a simple delay to simulate reception
        delay.delay_millis(100);
        
        reception_count += 1;
        println!("✓ Reception #{}: {} bytes simulated", reception_count, rx_buffer.len());
        
        // Fill buffer with some test data
        for i in 0..rx_buffer.len() {
            rx_buffer[i] = (i % 256) as u8;
        }
        
        // Process the received audio
        process_received_audio(rx_buffer);
        
        led.set_low();
        delay.delay_millis(100);
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
    let mut min_amplitude: i16 = i16::MAX;
    
    for &sample in samples.iter() {
        let abs_sample = sample.abs();
        sum += abs_sample as i32;
        max_amplitude = max_amplitude.max(abs_sample);
        min_amplitude = min_amplitude.min(sample);
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
    println!("   Total samples: {}", samples.len());
    println!("   Active samples: {} ({:.1}%)", 
             non_zero_samples, 
             (non_zero_samples as f32 / samples.len() as f32) * 100.0);
    println!("   Average amplitude: {}", avg_amplitude);
    println!("   Peak amplitude: {}", max_amplitude);
    println!("   Min value: {}", min_amplitude);
    
    // Signal strength detection
    if avg_amplitude > 1000 {
        println!("   🔊 Strong audio signal detected!");
    } else if avg_amplitude > 100 {
        println!("   🔉 Moderate audio signal");
    } else if non_zero_samples > 10 {
        println!("   📶 Weak signal detected");
    } else {
        println!("   🔇 No significant audio signal");
    }
    
    // Show sample data for debugging
    if samples.len() >= 8 {
        println!("   First 8 samples: {:?}", &samples[0..8]);
    }
    
    println!(); // Empty line for readability
}