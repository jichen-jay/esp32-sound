# I2S Waveform ASCII Simulation for ESP32-H2

Based on your ESP32-H2 I2S configuration (16 kHz sample rate, 16-bit stereo, 512 kHz BCLK), here are the expected waveforms you should see on your oscilloscope:

## Complete I2S Frame Overview

```
Time Scale: Each character ≈ 2 μs, Total frame = 62.5 μs

BCLK  ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
(512k)│ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │
      └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘

WS    ┌───────────────────────────────────┐                               ┌─
(16k) │           LEFT CHANNEL            │         RIGHT CHANNEL         │
      └───────────────────────────────────┘───────────────────────────────┘

DOUT  ──┐ ┌─┐ ┌───┐ ┌─┐ ┌─┐   ┌─┐ ┌───────┐ ┌─┐   ┌─┐ ┌─┐ ┌───┐ ┌─────────
      M │S│ │B│   │ │ │ │ │LSB│ │ │PADDING│ │M│   │ │ │ │ │   │ │
      S │B│ │ │   │ │ │ │ │   │ │ │   0   │ │S│   │ │ │ │ │   │ │
      B └─┘ └─┘   └─┘ └─┘ └───┘ └─┘       └─┘ └───┘ └─┘ └─┘   └─┘

      ←── 16 bits audio ──→←─ 16 pad ─→    ←── 16 bits audio ──→←─ 16 pad ─→
      ←─────── 32 BCLK cycles ──────→    ←─────── 32 BCLK cycles ──────→
```

## Detailed Single I2S Frame (Left + Right Channels)

```
Frame Duration: 62.5 μs (16 kHz sample rate)
BCLK Cycles per Frame: 64 (32 per channel)

Time:  0μs    15.6μs      31.25μs     46.9μs      62.5μs
       │       │           │           │           │
BCLK   ┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐
       │ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ │
       └─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘

WS     ┌───────────────────────────────────┐
       │         LEFT = 0                  │      RIGHT = 1
       └───────────────────────────────────┘─────────────────────────────────

DOUT   ──┐ ┌─┐ ┌───┐ ┌─┐ ┌─┐   ┌─┐ ┌─────┐ ┌─┐   ┌─┐ ┌─┐ ┌───┐ ┌─────────
Sample │1│0│1│0│1 1│0│0│0│...│0│0│0│0 0 0│1│0│...│1│0│1│0│1 1│0│0│0 0 0...
Data   └─┘ └─┘ └───┘ └─┘ └─┘   └─┘ └─────┘ └─┘   └─┘ └─┘ └───┘ └─────────
       MSB                 LSB   Padding    MSB                 LSB  Padding

Bit:   15 14 13 12 11..  1  0  x  x  x     15 14 13 12 11..  1  0  x  x  x
```

## Individual Signal Analysis

### BCLK (Bit Clock) - GPIO4
```
Frequency: 512 kHz
Period: 1.953 μs per cycle
Duty Cycle: 50%

Single BCLK cycle (1.953 μs):
Time: 0      976ns    1.953μs
      ┌──────┐
3.3V  │      │
      │      │
 0V   └──────┘
      ←─ High ─→←─ Low ─→
```

### WS (Word Select/Frame Sync) - GPIO5
```
Frequency: 16 kHz  
Period: 62.5 μs per frame
Duty Cycle: 50%

Complete WS cycle (62.5 μs):
Time: 0      31.25μs     62.5μs      93.75μs     125μs
      ┌──────────────────┐           ┌──────────────────┐
3.3V  │   LEFT CHANNEL   │           │   LEFT CHANNEL   │
      │                  │           │                  │
 0V   └──────────────────┘───────────┘──────────────────┘
      ←─────── Frame N ────────→←─────── Frame N+1 ──────→
                         ←─ RIGHT ─→
```

### DOUT (Data Output) - GPIO6
```
Sample Audio Data (16-bit signed):
Example: Left = 0x1A2B, Right = 0x3C4D

DOUT during one complete frame:
       ┌─┐   ┌─┐ ┌─┐   ┌─┐   ┌─┐       ┌─┐ ┌─┐ ┌─┐   ┌─┐ ┌─┐   ┌─┐
3.3V   │ │   │ │ │ │   │ │   │ │       │ │ │ │ │ │   │ │ │ │   │ │
       │ │   │ │ │ │   │ │   │ │       │ │ │ │ │ │   │ │ │ │   │ │
  0V   └─┘───┘ └─┘ └───┘ └───┘ └───────┘ └─┘ └─┘ └───┘ └─┘ └───┘ └─

Bit:   1 0 0 1 1 0 1 0 0 0 1 0 1 0 1 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
       ←─────── 0x1A2B (Left) ─────────→←──────── Padding ────────→

       3 1 1 0 0 1 0 0 0 1 0 0 1 1 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
       ←─────── 0x3C4D (Right) ────────→←──────── Padding ────────→
```

## Oscilloscope Display Simulation

### 3-Channel View (Recommended Setup)
```
Channel 1 (BCLK - Yellow):
┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐┌─┐
│ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ ││ │
└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘└─┘

Channel 2 (WS - Blue):
┌───────────────────────────────────┐
│         LEFT CHANNEL              │         RIGHT CHANNEL
└───────────────────────────────────┘───────────────────────────────────

Channel 3 (DOUT - Red):
──┐ ┌─┐ ┌───┐ ┌─┐ ┌─┐   ┌─┐ ┌─────┐ ┌─┐   ┌─┐ ┌─┐ ┌───┐ ┌─────────
  │ │ │ │   │ │ │ │ │   │ │ │     │ │ │   │ │ │ │ │   │ │
  └─┘ └─┘   └─┘ └─┘ └───┘ └─┘     └─┘ └───┘ └─┘ └─┘   └─┘

Time/Div: 10 μs/div
Voltage/Div: 1V/div
```

## Key Timing Relationships to Verify

### Critical Measurements
1. **BCLK Frequency**: Exactly 512 kHz (1.953 μs period)
2. **WS Frequency**: Exactly 16 kHz (62.5 μs period)  
3. **Data Setup Time**: DOUT stable 100ns before BCLK rising edge
4. **Frame Alignment**: WS transitions on BCLK falling edge
5. **Bit Count**: Exactly 32 BCLK cycles per channel (64 total per frame)

### Expected Signal Characteristics
- **Logic Levels**: 0V (low) to 3.3V (high)
- **Rise/Fall Times**: < 10ns for clean digital signals
- **Jitter**: < 1% of BCLK period (< 20ns)
- **Synchronization**: All transitions aligned to BCLK edges

This ASCII simulation shows exactly what you should observe on your oscilloscope when analyzing your ESP32-H2 I2S audio transmission. The actual Happy Birthday audio data will create varying patterns in the DOUT signal, but the timing relationships will remain constant.