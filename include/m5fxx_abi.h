/*
 * m5fxx_abi.h - C-ABI for connecting C/C++ M5Cardputer Firmware to m5fxx simulator
 *
 * This header defines the narrow, portable C interface between
 * native C/C++ firmware applications and the m5fxx Rust HAL simulator.
 */

#ifndef M5FXX_ABI_H
#define M5FXX_ABI_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define M5FXX_DISPLAY_WIDTH  240
#define M5FXX_DISPLAY_HEIGHT 135

/* Key Event Types */
typedef enum {
    M5FXX_KEY_PRESS = 1,
    M5FXX_KEY_RELEASE = 2,
} m5fxx_key_action_t;

/* Modifier Flags */
#define M5FXX_MOD_FN    (1 << 0)
#define M5FXX_MOD_SHIFT (1 << 1)
#define M5FXX_MOD_CTRL  (1 << 2)
#define M5FXX_MOD_OPT   (1 << 3)
#define M5FXX_MOD_ALT   (1 << 4)

/* Key Event Structure */
typedef struct {
    uint8_t row;          /* 0..3 */
    uint8_t col;          /* 0..13 */
    uint8_t modifiers;    /* M5FXX_MOD_* flags */
    char ascii_char;      /* Resolved ASCII char, or 0 if modifier/special */
    m5fxx_key_action_t action;
} m5fxx_key_event_t;

/* C/C++ Firmware Application Callbacks (Exported by Firmware) */
typedef struct {
    /* Called once upon initialization */
    void (*init)(void* user_data);

    /* Called periodically every frame (dt in seconds) */
    void (*update)(void* user_data, float dt);

    /* Called when key state changes */
    void (*on_key_event)(void* user_data, const m5fxx_key_event_t* event);

    /* Called on system reset */
    void (*reset)(void* user_data);
} m5fxx_firmware_interface_t;

/* Host / Simulator HAL Services (Imported by Firmware) */
typedef struct {
    /* Direct pointer to the 240x135 RGB565 framebuffer (51,840 bytes) */
    uint16_t* (*get_framebuffer)(void);

    /* Mark display dirty to trigger texture upload */
    void (*display_flush)(void);

    /* Monotonic time in milliseconds */
    uint64_t (*millis)(void);

    /* SD Card sandboxed file I/O */
    int32_t (*sd_read)(const char* path, uint8_t* out_buf, uint32_t max_len);
    int32_t (*sd_write)(const char* path, const uint8_t* data, uint32_t len);

    /* Logging */
    void (*log)(const char* msg);
} m5fxx_host_services_t;

#ifdef __cplusplus
}
#endif

#endif /* M5FXX_ABI_H */
