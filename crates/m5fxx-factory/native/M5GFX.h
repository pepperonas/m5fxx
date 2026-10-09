#pragma once
#include <lgfx/v1/LGFXBase.hpp>
#include <lgfx/v1/LGFX_Sprite.hpp>
using LGFX_Sprite = lgfx::LGFX_Sprite;
using lgfx::textdatum_t;
namespace m5gfx {
using lgfx::v1::delay;
using lgfx::v1::millis;
} // namespace m5gfx
class M5GFX : public LGFX_Sprite {
public:
  uint8_t brightness = 255;
  M5GFX() : LGFX_Sprite(nullptr) {}
  void setBrightness(uint8_t value) { brightness = value; }
  void begin() {
    setColorDepth(16);
    createSprite(240, 135);
  }
};
