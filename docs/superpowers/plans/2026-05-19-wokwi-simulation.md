# Wokwi Simulation Setup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Set up Wokwi CLI simulation for the plant-saver ESP32 project with local development workflow and web sharing capability.

**Architecture:** Add a `wokwi/` directory with `diagram.json` (ESP32 + potentiometer + 2 LEDs) and `wokwi.toml` (ELF path config). No code changes needed — the firmware already works with the simulated hardware.

**Tech Stack:** Wokwi CLI, ESP32 DevKit V1 simulation, potentiometer (ADC), LEDs

---

### Task 1: Create wokwi.toml

**Files:**
- Create: `wokwi/wokwi.toml`

- [ ] **Step 1: Create wokwi directory**

```bash
mkdir -p wokwi
```

- [ ] **Step 2: Create wokwi.toml**

Create `wokwi/wokwi.toml`:

```toml
[wokwi]
version = 1
firmware = "../target/xtensa-esp32-espidf/release/plant-saver"
elf = "../target/xtensa-esp32-espidf/release/plant-saver"
```

- [ ] **Step 3: Commit**

```bash
git add wokwi/wokwi.toml
git commit -m "feat: add wokwi simulation config"
```

---

### Task 2: Create diagram.json

**Files:**
- Create: `wokwi/diagram.json`

- [ ] **Step 1: Create diagram.json**

Create `wokwi/diagram.json`:

```json
{
  "version": 1,
  "author": "Guillaume Lamanda",
  "editor": "wokwi",
  "parts": [
    {
      "type": "board-esp32-devkit-v1",
      "id": "esp",
      "top": 0,
      "left": 100,
      "attrs": {}
    },
    {
      "type": "potentiometer",
      "id": "moisture_pot",
      "top": -100,
      "left": -150,
      "attrs": { "value": "50" }
    },
    {
      "type": "led",
      "id": "moisture_led",
      "top": -50,
      "left": 400,
      "attrs": { "color": "red" }
    },
    {
      "type": "led",
      "id": "luminosity_led",
      "top": 50,
      "left": 400,
      "attrs": { "color": "green" }
    },
    {
      "type": "resistor",
      "id": "r1",
      "top": -50,
      "left": 350,
      "attrs": { "resistance": "220" }
    },
    {
      "type": "resistor",
      "id": "r2",
      "top": 50,
      "left": 350,
      "attrs": { "resistance": "220" }
    }
  ],
  "connections": [
    ["moisture_pot:SIG", "esp:13", "orange", ["v0"]],
    ["moisture_pot:VCC", "esp:3V3", "red", ["v0"]],
    ["moisture_pot:GND", "esp:GND.1", "black", ["v0"]],

    ["esp:12", "r1:1", "red", ["v0"]],
    ["r1:2", "moisture_led:A", "red", ["v0"]],
    ["moisture_led:C", "esp:GND.2", "black", ["v0"]],

    ["esp:4", "r2:1", "green", ["v0"]],
    ["r2:2", "luminosity_led:A", "green", ["v0"]],
    ["luminosity_led:C", "esp:GND.3", "black", ["v0"]]
  ]
}
```

Pin mapping:
- Potentiometer SIG → GPIO13 (ADC2_CH2, moisture sensor input)
- Potentiometer VCC → 3V3
- Potentiometer GND → GND
- GPIO12 → 220Ω resistor → red LED anode (moisture alert)
- GPIO4 → 220Ω resistor → green LED anode (luminosity indicator)
- Both LED cathodes → GND

- [ ] **Step 2: Commit**

```bash
git add wokwi/diagram.json
git commit -m "feat: add wokwi circuit diagram"
```

---

### Task 3: Update README with Wokwi instructions

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add Wokwi section to README**

Add after the `## Commands` section in `README.md`:

```markdown
## Simulation (Wokwi)

- Install: `pip install wokwi-cli`
- Build firmware: `cargo build --release`
- Run simulation: `wokwi-cli` from repo root
- Adjust the potentiometer to simulate moisture changes
- Web project: https://wokwi.com/projects/323845300685898322
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add wokwi simulation instructions"
```
