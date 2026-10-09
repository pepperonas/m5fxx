#pragma once
#include "M5GFX.h"
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <ctime>
namespace m5 {
struct Button_Class {
  bool pressed = false, previous = false;
  bool wasPressed() { return pressed && !previous; }
  bool wasClicked() { return !pressed && previous; }
  bool isPressed() { return pressed; }
  bool wasReleased() { return wasClicked(); }
};
struct Speaker_Class {
  bool enabled = true;
  int volume = 90;
  unsigned long playing_until = 0;
  struct Config {
    int sample_rate = 16000;
  };
  Config config() { return {}; }
  void config(Config) {}
  void begin() { enabled = true; }
  void end() { enabled = false; }
  bool isEnabled() { return enabled; }
  void stop() { playing_until = 0; }
  void setVolume(int v) { volume = v; }
  int getVolume() { return volume; }
  bool isPlaying() { return enabled && lgfx::millis() < playing_until; }
  template <class... T> void tone(T...) {}
  template <class... T> void playWav(T...) {
    playing_until = lgfx::millis() + 300;
  }
  template <class T> void playRaw(T *, size_t count, int rate, bool) {
    playing_until = lgfx::millis() + count * 1000 / std::max(1, rate);
  }
};
struct Mic_Class {
  bool enabled = false;
  float amplitude = 0;
  uint32_t phase = 0;
  struct Config {
    int magnification = 128, noise_filter_level = 2;
  };
  Config config() { return {}; }
  void config(Config) {}
  bool isEnabled() { return enabled; }
  void begin() { enabled = true; }
  void end() { enabled = false; }
  bool isRecording() { return false; }
  template <class T> bool record(T *data, size_t count, int) {
    for (size_t i = 0; i < count; i++) {
      data[i] = static_cast<T>(std::sin((phase++ % 128) * 6.2831853 / 128) *
                               amplitude * 12000);
    }
    return true;
  }
};
struct IMU_Class {
  struct Vec {
    float x = 0, y = 0, z = 0;
  };
  struct imu_data_t {
    Vec accel{0, 0, 1}, gyro;
  };
  imu_data_t data;
  void begin() {}
  bool update() { return true; }
  imu_data_t getImuData() { return data; }
};
using imu_data_t = IMU_Class::imu_data_t;
} // namespace m5
struct Unified {
  M5GFX Display;
  m5::Button_Class BtnA;
  m5::Speaker_Class Speaker;
  m5::Mic_Class Mic;
  m5::IMU_Class Imu;
};
extern Unified M5;
inline int esp_reset_reason() { return 1; }
#define ESP_RST_POWERON 1
inline void esp_restart() {}
