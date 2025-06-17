//! ESP32 I2S Audio Communication System
//! Supports both sending and receiving audio data between two ESP32 devices
//! Compatible with ESP32-S3 based devices (ESP32-DOWDQ6 v3 and T-Embed)

#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    dma::{Dma, DmaPriority, DmaTxBuf, DmaRxBuf},
    dma_buffers,
    gpio::{Io, Level, Output},
    i2s::master::{DataFormat, I2s, Standard},
    prelude::*,
    system::SystemControl,
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_println::println;

// Include the generated audio data
// You'll need to run the Python script first to generate this file
include!("happy_birthday_audio.rs");

// I2S Configuration
const I2S_SAMPLE_RATE: u32 = SAMPLE_RATE;
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes - must be large enough for audio chunks
const TX_BUFFER_SIZE: usize = 4096;
const RX_BUFFER_SIZE: usize = 4096;

// Device mode selection - change this to switch between sender/receiver
const DEVICE_MODE: DeviceMode = DeviceMode::Sender;

#[derive(Clone, Copy)]
enum DeviceMode {
    Sender,
    Receiver,
}

#[entry]
fn main() -> ! {
    let system = SystemControl::new();
    let mut peripheral_clock_control = system.peripheral_clock_control;
    let clocks = esp_hal::clock::ClockControl::max(&mut peripheral_clock_control)
        .freeze();

    let delay = Delay::new(&clocks);
    let io = Io::new(esp_hal::gpio::Gpio::new(), io::Interrupt::IO);

    println!("ESP32 I2S Audio Communication System");
    println!("Device mode: {:?}", DEVICE_MODE);

    // Initialize DMA
    let dma = Dma::new(&mut peripheral_clock_control.dma);
    let dma_channel = dma.channel0;

    // Create DMA buffers
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = 
        dma_buffers!(RX_BUFFER_SIZE, TX_BUFFER_SIZE);

    // Configure I2S pins based on your hardware
    // Adjust these pin assignments for your specific boards
    let i2s = I2s::new(
        esp_hal::peripheral::Peripheral::I2S0,
        I2S_STANDARD,
        I2S_DATA_FORMAT,
        Rate::from_hz(I2S_SAMPLE_RATE),
        dma_channel,
    );

    // Configure pins - adjust these for your hardware
    // For ESP32-S3 T-Embed and similar boards
    match DEVICE_MODE {
        DeviceMode::Sender => {
            run_sender(i2s, tx_buffer, tx_descriptors, io, delay);
        }
        DeviceMode::Receiver => {
            run_receiver(i2s, rx_buffer, rx_descriptors, io, delay);
        }
    }
}

fn run_sender(
    i2s: I2s<esp_hal::Blocking>,
    tx_buffer: &'static mut [u8],
    tx_descriptors: &'static mut [esp_hal::dma::DmaDescriptor],
    io: Io,
    mut delay: Delay,
) -> ! {
    println!("Initializing as SENDER");

    // Configure I2S TX pins
    let i2s_tx = i2s.i2s_tx
        .with_bclk(io.pins.gpio1)      // Bit clock
        .with_ws(io.pins.gpio2)        // Word select
        .with_dout(io.pins.gpio3)      // Data out
        .build(tx_descriptors);

    // Status LED
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    println!("Starting audio transmission...");
    println!("Audio data size: {} bytes", HAPPY_BIRTHDAY_AUDIO.len());

    loop {
        led.set_high();
        
        // Copy audio data to DMA buffer in chunks
        let mut offset = 0;
        while offset < HAPPY_BIRTHDAY_AUDIO.len() {
            let chunk_size = (HAPPY_BIRTHDAY_AUDIO.len() - offset).min(tx_buffer.len());
            
            // Copy audio chunk to DMA buffer
            tx_buffer[..chunk_size].copy_from_slice(
                &HAPPY_BIRTHDAY_AUDIO[offset..offset + chunk_size]
            );
            
            // Transmit chunk
            println!("Transmitting audio chunk: {} bytes at offset {}", chunk_size, offset);
            
            if let Err(e) = i2s_tx.write_words(
                unsafe { 
                    core::slice::from_raw_parts(
                        tx_buffer.as_ptr() as *const u16,
                        chunk_size / 2
                    )
                }
            ) {
                println!("I2S write error: {:?}", e);
            }
            
            offset += chunk_size;
            delay.delay_millis(10); // Small delay between chunks
        }
        
        led.set_low();
        println!("Audio transmission complete, waiting 2 seconds...");
        delay.delay_millis(2000);
    }
}

fn run_receiver(
    i2s: I2s<esp_hal::Blocking>,
    rx_buffer: &'static mut [u8],
    rx_descriptors: &'static mut [esp_hal::dma::DmaDescriptor],
    io: Io,
    mut delay: Delay,
) -> ! {
    println!("Initializing as RECEIVER");

    // Configure I2S RX pins
    let mut i2s_rx = i2s.i2s_rx
        .with_bclk(io.pins.gpio1)      // Bit clock
        .with_ws(io.pins.gpio2)        // Word select  
        .with_din(io.pins.gpio3)       // Data in
        .build(rx_descriptors);

    // Status LED
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    println!("Starting audio reception...");

    loop {
        led.set_high();
        println!("Waiting for audio data...");
        
        // Receive audio data
        if let Err(e) = i2s_rx.read_words(
            unsafe {
                core::slice::from_raw_parts_mut(
                    rx_buffer.as_mut_ptr() as *mut u16,
                    rx_buffer.len() / 2
                )
            }
        ) {
            println!("I2S read error: {:?}", e);
        } else {
            println!("Received {} bytes of audio data", rx_buffer.len());
            
            // Process received audio data here
            // For example, you could:
            // 1. Store it for playback
            // 2. Analyze the audio
            // 3. Forward it to another device
            // 4. Apply audio processing
            
            process_received_audio(rx_buffer);
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
    
    for &sample in samples.iter() {
        sum += sample.abs() as i32;
        max_amplitude = max_amplitude.max(sample.abs());
    }
    
    let avg_amplitude = if samples.len() > 0 { 
        sum / samples.len() as i32 
    } else { 
        0 
    };
    
    println!("Audio stats - Avg amplitude: {}, Max amplitude: {}", 
             avg_amplitude, max_amplitude);
    
    // Detect if there's meaningful audio content
    if avg_amplitude > 1000 {
        println!("Strong audio signal detected!");
    } else if avg_amplitude > 100 {
        println!("Weak audio signal detected");
    } else {
        println!("No significant audio signal");
    }
}

// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Panic occurred: {:?}", info);
    loop {}
}

use core::panic::PanicInfo;