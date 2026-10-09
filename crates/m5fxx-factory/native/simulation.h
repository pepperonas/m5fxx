#pragma once
#include <cstdint>
struct SimulationInputs {
  float accel[3], gyro[3], audio_amplitude;
  uint64_t sd_bytes;
  uint8_t battery, wifi, ble, usb, sd, cap;
};
