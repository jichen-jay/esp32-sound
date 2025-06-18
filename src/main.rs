//! ESP32-H2 I2S FM-Style Pattern Generator
//! Creates FM-like patterns using digital square waves through I2S
//! GPIO4: BCLK, GPIO5: WS, GPIO12: DOUT

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

const SAMPLE_RATE: u32 = 16000; // Higher sample rate for better FM resolution
const I2S_DATA_FORMAT: DataFormat = DataFormat::Data16Channel16;
const I2S_STANDARD: Standard = Standard::Philips;

// DMA Buffer sizes
const TX_BUFFER_SIZE: usize = 512;
const RX_BUFFER_SIZE: usize = 256;

#[entry]
fn main() -> ! {
    println!("📻 ESP32-H2 I2S FM-STYLE PATTERN GENERATOR 📻");
    println!("🎵 Creates FM-like patterns using digital square waves!");
    
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let delay = Delay::new(&clocks);
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Status LED
    let mut led = Output::new(io.pins.gpio8, Level::Low);

    // LED startup
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

    let (_rx_buffer, rx_descriptors, _tx_buffer, tx_descriptors) = 
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

    // Configure I2S TX pins
    let bclk = io.pins.gpio4;
    let ws = io.pins.gpio5;
    let dout = io.pins.gpio12;
    
    let mut i2s_tx = i2s.i2s_tx
        .with_bclk(bclk)
        .with_ws(ws)
        .with_dout(dout)
        .build();
    
    println!("✅ I2S TX Configuration Complete:");
    println!("   🔌 BCLK: GPIO4 (Bit Clock)");
    println!("   🔌 WS:   GPIO5 (Word Select/Frame Sync)"); 
    println!("   🔌 DOUT: GPIO12 (FM-Style Data Out)");
    println!("   📊 Sample Rate: {} Hz", SAMPLE_RATE);
    println!("   🎼 Format: 16-bit, Philips I2S");
    println!();
    println!("📻 FM-STYLE OSCILLOSCOPE VIEWING:");
    println!("   📺 Connect oscilloscope to GPIO12 (DOUT)");
    println!("   📏 Time scale: 10ms/div (see FM frequency changes)");
    println!("   📏 Time scale: 100ms/div (see complete patterns)");
    println!("   📈 Voltage scale: 1V/div");
    println!("   ⚡ Trigger: Rising edge");
    println!("   🎯 Look for frequency modulation patterns!");
    println!();
    println!("🎵 FM Pattern Sequence:");
    println!("   📻 Frequency Sweep (Low→High→Low)");
    println!("   📡 AM-like Modulation (Amplitude bursts)");
    println!("   🌊 Frequency Wobble (Back and forth)");
    println!("   📊 Step Frequency (Digital frequency steps)");
    println!("   💫 Chirp Signal (Quick frequency sweep)");
    println!("   🎶 Musical Scale (Note progression)");
    println!("   📢 SOS Morse in FM (Emergency signal)");
    println!();

    let mut cycle_count = 0;

    loop {
        cycle_count += 1;
        
        println!("📻 === FM PATTERN CYCLE #{} === 📻", cycle_count);
        
        // FM Pattern 1: Frequency Sweep (Low to High to Low)
        {
            led.set_high();
            println!("🎵 FM Pattern 1/7: Frequency Sweep");
            println!("   📻 Frequency gradually increases then decreases");
            
            // Create frequency sweep using varying square wave patterns
            for sweep in 0..40 {
                let mut pattern = [0u16; 32];
                
                // Calculate frequency: low at start/end, high in middle
                let freq_factor = if sweep < 20 {
                    sweep + 1  // Increasing frequency
                } else {
                    41 - sweep // Decreasing frequency
                };
                
                // Create square wave with varying frequency
                let half_period = 32 / (freq_factor / 2).max(1);
                for i in 0..32 {
                    pattern[i] = if (i / half_period) % 2 == 0 {
                        0x8000  // High
                    } else {
                        0x0000  // Low
                    };
                }
                
                match i2s_tx.write(&pattern) {
                    Ok(_) => {
                        if sweep % 10 == 0 {
                            println!("   📊 Sweep progress: {}%", (sweep * 100) / 40);
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Error: {:?}", e);
                        break;
                    }
                }
                delay.delay_millis(50);
            }
            
            led.set_low();
            println!("   ✅ Frequency sweep complete");
            delay.delay_millis(300);
        }
        
        // FM Pattern 2: AM-like Modulation (Amplitude Bursts)
        {
            led.set_high();
            println!("🎵 FM Pattern 2/7: AM-like Amplitude Modulation");
            println!("   📡 Square wave with varying amplitude envelopes");
            
            for burst in 0..20 {
                let mut pattern = [0u16; 32];
                
                // Create envelope: amplitude varies in a wave pattern
                let envelope = if burst < 5 {
                    (burst as f32) / 5.0  // Rising
                } else if burst < 15 {
                    1.0  // Peak
                } else {
                    (20 - burst) as f32 / 5.0  // Falling
                };
                let amplitude = (envelope * 32767.0) as u16;
                
                // Create square wave with modulated amplitude
                for i in 0..32 {
                    pattern[i] = if i % 4 < 2 {
                        amplitude  // High with envelope
                    } else {
                        0x0000     // Low
                    };
                }
                
                match i2s_tx.write(&pattern) {
                    Ok(_) => {
                        if burst % 5 == 0 {
                            println!("   📊 AM burst: {}/20 (envelope: {:.1}%)", burst + 1, envelope * 100.0);
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Error: {:?}", e);
                        break;
                    }
                }
                delay.delay_millis(75);
            }
            
            led.set_low();
            println!("   ✅ AM modulation complete");
            delay.delay_millis(300);
        }
        
        // FM Pattern 3: Frequency Wobble (Back and Forth)
        {
            led.set_high();
            println!("🎵 FM Pattern 3/7: Frequency Wobble");
            println!("   🌊 Frequency oscillates back and forth");
            
            for wobble in 0..30 {
                let mut pattern = [0u16; 32];
                
                // Create wobbling frequency (triangle wave frequency modulation)
                let wobble_factor = if wobble < 8 {
                    3 + wobble  // Rising frequency
                } else if wobble < 23 {
                    11  // Peak frequency
                } else {
                    33 - wobble  // Falling frequency
                } as usize;
                let period = (32 / wobble_factor).max(2);
                
                for i in 0..32 {
                    pattern[i] = if (i / period) % 2 == 0 {
                        0x8000  // High
                    } else {
                        0x0000  // Low
                    };
                }
                
                match i2s_tx.write(&pattern) {
                    Ok(_) => {
                        if wobble % 8 == 0 {
                            println!("   🌊 Wobble cycle: {}/30", wobble + 1);
                        }
                    }
                    Err(e) => {
                        println!("   ❌ Error: {:?}", e);
                        break;
                    }
                }
                delay.delay_millis(60);
            }
            
            led.set_low();
            println!("   ✅ Frequency wobble complete");
            delay.delay_millis(300);
        }
        
        // FM Pattern 4: Step Frequency (Digital Steps)
        {
            led.set_high();
            println!("🎵 FM Pattern 4/7: Step Frequency Changes");
            println!("   📊 Discrete frequency steps (digital tuning)");
            
            let frequencies = [2, 4, 6, 8, 12, 16, 8, 4]; // Different step frequencies
            
            for (step, &freq) in frequencies.iter().enumerate() {
                let mut pattern = [0u16; 32];
                let period = (32 / freq).max(1);
                
                for i in 0..32 {
                    pattern[i] = if (i / period) % 2 == 0 {
                        0x8000  // High
                    } else {
                        0x0000  // Low
                    };
                }
                
                println!("   📻 Step {}: Frequency {} (period {})", step + 1, freq, period);
                
                // Repeat each frequency step multiple times
                for repeat in 0..8 {
                    match i2s_tx.write(&pattern) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("   ❌ Error: {:?}", e);
                            break;
                        }
                    }
                    delay.delay_millis(40);
                }
                
                delay.delay_millis(100); // Pause between steps
            }
            
            led.set_low();
            println!("   ✅ Step frequency complete");
            delay.delay_millis(300);
        }
        
        // FM Pattern 5: Chirp Signal (Quick Frequency Sweep)
        {
            led.set_high();
            println!("🎵 FM Pattern 5/7: Chirp Signal");
            println!("   💫 Rapid frequency sweep (radar-like chirp)");
            
            for chirp in 0..3 { // 3 chirp cycles
                println!("   💫 Chirp {}/3", chirp + 1);
                
                // Quick frequency sweep from low to high
                for freq_step in 1..=16 {
                    let mut pattern = [0u16; 32];
                    let period = (32 / freq_step).max(1);
                    
                    for i in 0..32 {
                        pattern[i] = if (i / period) % 2 == 0 {
                            0x8000  // High
                        } else {
                            0x0000  // Low
                        };
                    }
                    
                    match i2s_tx.write(&pattern) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("   ❌ Error: {:?}", e);
                            break;
                        }
                    }
                    delay.delay_millis(20); // Quick sweep
                }
                
                delay.delay_millis(200); // Pause between chirps
            }
            
            led.set_low();
            println!("   ✅ Chirp signal complete");
            delay.delay_millis(300);
        }
        
        // FM Pattern 6: Musical Scale (Note Progression)
        {
            led.set_high();
            println!("🎵 FM Pattern 6/7: Musical Scale");
            println!("   🎶 Frequency steps mimicking musical notes");
            
            // Musical scale frequencies (simplified as periods)
            let notes = [16, 14, 12, 11, 10, 9, 8, 7]; // Descending scale
            let note_names = ["C", "D", "E", "F", "G", "A", "B", "C"];
            
            for (note_idx, &note_period) in notes.iter().enumerate() {
                let mut pattern = [0u16; 32];
                
                for i in 0..32 {
                    pattern[i] = if (i / note_period) % 2 == 0 {
                        0x8000  // High
                    } else {
                        0x0000  // Low
                    };
                }
                
                println!("   🎵 Note {}: {} (period {})", note_idx + 1, note_names[note_idx], note_period);
                
                // Play each note
                for repeat in 0..6 {
                    match i2s_tx.write(&pattern) {
                        Ok(_) => {},
                        Err(e) => {
                            println!("   ❌ Error: {:?}", e);
                            break;
                        }
                    }
                    delay.delay_millis(80);
                }
                
                delay.delay_millis(50); // Brief pause between notes
            }
            
            led.set_low();
            println!("   ✅ Musical scale complete");
            delay.delay_millis(300);
        }
        
        // FM Pattern 7: SOS Morse in FM
        {
            led.set_high();
            println!("🎵 FM Pattern 7/7: SOS Morse Code in FM");
            println!("   📢 Emergency signal using frequency modulation");
            
            // SOS: ... --- ... (3 dots, 3 dashes, 3 dots)
            let sos_pattern = [
                (8, 4),   // S: dot (high freq, short)
                (8, 4),   // S: dot  
                (8, 4),   // S: dot
                (0, 8),   // Gap
                (4, 12),  // O: dash (low freq, long)
                (4, 12),  // O: dash
                (4, 12),  // O: dash  
                (0, 8),   // Gap
                (8, 4),   // S: dot
                (8, 4),   // S: dot
                (8, 4),   // S: dot
            ];
            
            for sos_cycle in 0..2 {
                println!("   📢 SOS transmission {}/2", sos_cycle + 1);
                
                for (freq, duration) in sos_pattern.iter() {
                    if *freq == 0 {
                        // Silence (gap)
                        let silence = [0u16; 32];
                        for _ in 0..*duration {
                            match i2s_tx.write(&silence) {
                                Ok(_) => {},
                                Err(e) => {
                                    println!("   ❌ Error: {:?}", e);
                                    break;
                                }
                            }
                            delay.delay_millis(50);
                        }
                    } else {
                        // Tone with specific frequency
                        let mut pattern = [0u16; 32];
                        let period = 32 / freq;
                        
                        for i in 0..32 {
                            pattern[i] = if (i / period) % 2 == 0 {
                                0x8000  // High
                            } else {
                                0x0000  // Low
                            };
                        }
                        
                        for _ in 0..*duration {
                            match i2s_tx.write(&pattern) {
                                Ok(_) => {},
                                Err(e) => {
                                    println!("   ❌ Error: {:?}", e);
                                    break;
                                }
                            }
                            delay.delay_millis(50);
                        }
                    }
                }
                
                delay.delay_millis(1000); // Long pause between SOS cycles
            }
            
            led.set_low();
            println!("   ✅ SOS transmission complete");
            delay.delay_millis(300);
        }
        
        println!("✅ Complete FM-style pattern cycle transmitted!");
        println!("   📻 All 7 FM patterns sent via I2S");
        println!("   🎵 Patterns visible as frequency modulation on GPIO12");
        println!("   📊 Total cycle duration: ~25 seconds");
        println!("   🔍 Observe different FM characteristics:");
        println!("      📻 Frequency sweeps (smooth changes)");
        println!("      📡 Amplitude modulation (burst patterns)");  
        println!("      🌊 Frequency wobbling (oscillation)");
        println!("      📊 Digital frequency steps");
        println!("      💫 Chirp signals (radar-like)");
        println!("      🎶 Musical note progression");
        println!("      📢 Morse code in FM");
        
        if cycle_count % 2 == 0 {
            println!("🎉 FM Cycle #{} complete - Check oscilloscope for patterns! 📻", cycle_count);
        }
        
        println!("⏳ Next FM cycle in 3 seconds...\n");
        delay.delay_millis(3000);
    }
}