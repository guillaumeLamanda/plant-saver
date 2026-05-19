# Wokwi Simulation Setup

## Context

Plant-saver is an ESP32 Rust firmware (esp-idf-svc) that reads a capacitive moisture sensor via ADC2 and drives two LEDs. We need a local simulation environment for development without physical hardware, plus the ability to share simulations via the web.

## Approach

**B: Wokwi CLI local + web-only sharing** — local simulation with `wokwi-cli`, no CI integration. Web sharing via manual upload to wokwi.com.

## Repository Structure

```
wokwi/
  diagram.json    # ESP32 + potentiometer (moisture) + 2 LEDs
  wokwi.toml      # Points to built ELF
```

## Diagram Components

| Component    | Wokwi ID               | ESP32 Pin        | Notes                                      |
|-------------|------------------------|------------------|---------------------------------------------|
| ESP32 DevKit V1 | `board-esp32-devkit-v1` | —              | Default board                               |
| Potentiometer   | `potentiometer`         | GPIO13 (ADC2_CH2) | Simulates moisture sensor analog output   |
| LED (red)       | `led`                   | GPIO12           | Moisture alert                              |
| LED (green)     | `led`                   | GPIO4            | Luminosity indicator                        |
| Resistor 220Ω   | `resistor`              | In series with each LED | Current limiting                    |

A potentiometer replaces the capacitive moisture sensor. Wokwi has no moisture sensor component — a pot on the ADC pin is the standard workaround, matching the 0–4095 ADC range the firmware expects.

## wokwi.toml

```toml
[wokwi]
version = 1
firmware = "../target/xtensa-esp32-espidf/release/plant-saver"
elf = "../target/xtensa-esp32-espidf/release/plant-saver"
```

For debug builds, swap `release` → `debug`.

## Local Workflow

1. Build: `cargo build --release`
2. Simulate: `wokwi-cli` from repo root (reads `wokwi/wokwi.toml`)
3. Interact: adjust potentiometer in Wokwi UI, observe LED state + serial output

## Web Sharing

- Keep the existing Wokwi project link from README (https://wokwi.com/projects/323845300685898322)
- When firmware changes, manually re-upload the ELF to the web project
- Diagram changes: copy `wokwi/diagram.json` content to the web editor

## Wokwi CLI Installation

- Install via: `pip install wokwi-cli` or download from GitHub releases
- Requires a Wokwi account + license (free tier available)
- Config stored at `~/.wokwi/`

## Out of Scope

- CI/CD integration with wokwi-cli (deferred — would require a Wokwi license token as a GitHub secret)
- Simulation of WiFi/Matter features (not yet in firmware)
- Automated diagram sync with web project
