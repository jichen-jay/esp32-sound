#!/usr/bin/env python3
"""
Generate Happy Birthday audio data for ESP32 I2S
Creates a simple sine wave melody at 16kHz sample rate
"""

import math
import struct
import wave

# Song configuration
SAMPLE_RATE = 16000
DURATION = 10.0  # seconds
AMPLITUDE = 0.3  # Moderate volume to avoid clipping

# Note frequencies (in Hz) - Happy Birthday melody
# C4=261.63, D4=293.66, E4=329.63, F4=349.23, G4=392.00, A4=440.00, B4=493.88, C5=523.25
NOTES = {
    'C4': 261.63, 'D4': 293.66, 'E4': 329.63, 'F4': 349.23,
    'G4': 392.00, 'A4': 440.00, 'B4': 493.88, 'C5': 523.25,
    'REST': 0.0
}

# Happy Birthday melody with timing (note, duration_in_beats)
# Total should be approximately 10 seconds at 120 BPM
MELODY = [
    ('C4', 0.75), ('C4', 0.25), ('D4', 1.0), ('C4', 1.0), ('F4', 1.0), ('E4', 2.0),
    ('C4', 0.75), ('C4', 0.25), ('D4', 1.0), ('C4', 1.0), ('G4', 1.0), ('F4', 2.0),
    ('C4', 0.75), ('C4', 0.25), ('C5', 1.0), ('A4', 1.0), ('F4', 1.0), ('E4', 1.0), ('D4', 2.0),
    ('B4', 0.75), ('B4', 0.25), ('A4', 1.0), ('F4', 1.0), ('G4', 1.0), ('F4', 2.0)
]

def generate_tone(frequency, duration, sample_rate, amplitude):
    """Generate a sine wave tone"""
    samples = int(sample_rate * duration)
    audio_data = []
    
    for i in range(samples):
        if frequency == 0:  # Rest
            sample = 0
        else:
            # Generate sine wave with envelope to avoid clicks
            t = i / sample_rate
            envelope = 1.0
            if duration > 0.1:  # Apply envelope for longer notes
                fade_time = min(0.05, duration / 4)  # 50ms or 1/4 note duration
                if t < fade_time:
                    envelope = t / fade_time
                elif t > duration - fade_time:
                    envelope = (duration - t) / fade_time
            
            sample = amplitude * envelope * math.sin(2 * math.pi * frequency * t)
        
        # Convert to 16-bit signed integer
        sample_int = int(sample * 32767)
        sample_int = max(-32768, min(32767, sample_int))  # Clamp to 16-bit range
        audio_data.append(sample_int)
    
    return audio_data

def generate_happy_birthday():
    """Generate the complete Happy Birthday song"""
    bpm = 120
    beat_duration = 60.0 / bpm  # Duration of one beat in seconds
    
    all_audio_data = []
    
    for note, beats in MELODY:
        frequency = NOTES[note]
        duration = beats * beat_duration
        tone_data = generate_tone(frequency, duration, SAMPLE_RATE, AMPLITUDE)
        all_audio_data.extend(tone_data)
        
        # Add small gap between notes
        gap_data = generate_tone(0, 0.05, SAMPLE_RATE, 0)
        all_audio_data.extend(gap_data)
    
    # Pad or trim to exactly 10 seconds
    target_samples = int(SAMPLE_RATE * DURATION)
    if len(all_audio_data) < target_samples:
        # Pad with silence
        all_audio_data.extend([0] * (target_samples - len(all_audio_data)))
    elif len(all_audio_data) > target_samples:
        # Trim to exact length
        all_audio_data = all_audio_data[:target_samples]
    
    return all_audio_data

def save_wav_file(audio_data, filename):
    """Save audio data as WAV file"""
    with wave.open(filename, 'w') as wav_file:
        wav_file.setnchannels(1)  # Mono
        wav_file.setsampwidth(2)  # 16-bit
        wav_file.setframerate(SAMPLE_RATE)
        
        # Convert to bytes
        for sample in audio_data:
            wav_file.writeframes(struct.pack('<h', sample))

def generate_rust_array(audio_data, filename):
    """Generate Rust array for embedding in code"""
    with open(filename, 'w') as f:
        f.write("// Happy Birthday audio data - 16kHz, 16-bit, mono, 10 seconds\n")
        f.write("// Generated automatically - do not edit\n\n")
        f.write("#[allow(dead_code)]\n")
        f.write("pub const HAPPY_BIRTHDAY_AUDIO: &[u8] = &[\n")
        
        # Convert 16-bit samples to little-endian bytes
        bytes_data = []
        for sample in audio_data:
            # Convert to little-endian 16-bit
            bytes_data.extend(struct.pack('<h', sample))
        
        # Write bytes in rows of 16
        for i in range(0, len(bytes_data), 16):
            row = bytes_data[i:i+16]
            hex_values = [f"0x{b:02x}" for b in row]
            f.write("    " + ", ".join(hex_values))
            if i + 16 < len(bytes_data):
                f.write(",")
            f.write("\n")
        
        f.write("];\n\n")
        f.write(f"pub const SAMPLE_RATE: u32 = {SAMPLE_RATE};\n")
        f.write(f"pub const AUDIO_LENGTH_SECONDS: f32 = {DURATION};\n")
        f.write(f"pub const AUDIO_SAMPLES: usize = {len(audio_data)};\n")

def main():
    print("Generating Happy Birthday audio data...")
    audio_data = generate_happy_birthday()
    
    print(f"Generated {len(audio_data)} samples ({len(audio_data)/SAMPLE_RATE:.1f} seconds)")
    
    # Save as WAV file for testing
    save_wav_file(audio_data, "happy_birthday_16khz.wav")
    print("Saved WAV file: happy_birthday_16khz.wav")
    
    # Generate Rust array
    generate_rust_array(audio_data, "happy_birthday_audio.rs")
    print("Generated Rust audio data: happy_birthday_audio.rs")
    
    print("\nAudio specifications:")
    print(f"- Sample rate: {SAMPLE_RATE} Hz")
    print(f"- Duration: {DURATION} seconds")
    print(f"- Bit depth: 16-bit signed")
    print(f"- Channels: Mono")
    print(f"- File size: {len(audio_data) * 2} bytes")

if __name__ == "__main__":
    main()