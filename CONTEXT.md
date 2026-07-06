# Domain glossary

This file defines the domain language used in the plant-saver project. It is a glossary — no implementation details, no decisions, no code.

## Soil moisture

The water content of the soil around the plant. Expressed as a percentage: 0% is completely dry, 100% is fully saturated.

## SoilMoistureLevel

A percentage value (0–100) representing the current soil moisture. The canonical output of the soil-moisture reading module.

## SoilMoisture

The module that reads the soil moisture sensor and produces a `SoilMoistureLevel`. It owns calibration, unit conversion, and clamping to the 0–100 range. It is not a hardware driver; it sits behind an ADC adapter seam.

## ADC adapter

A seam that supplies raw voltage readings from an analog-to-digital converter. Production implementation reads the ESP32 ADC; test implementations return fixed values. The `SoilMoisture` module depends on this adapter, not on the hardware directly.

## Calibration

The mapping from raw ADC readings to the physical dry and wet extremes used by `SoilMoisture`. Starts from configured bounds and can adapt automatically as the sensor observes new minimum and maximum readings.
