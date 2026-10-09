// Adapted from the pinned MIT-licensed M5Stack keyboard.cpp.
#include <hal/keyboard.h>
struct KeyValue_t {
    const char*        firstName;
    const KeScanCode_t firstKeyCode;
    const char*        secondName;
    const KeScanCode_t secondKeyCode;
    const char*        fnName;           // nullptr = no Fn override
    const KeScanCode_t fnKeyCode;        // KEY_NONE = no Fn override
    const uint8_t      fnExtraModifiers; // extra HID modifier bits injected for this Fn key
};

// clang-format off
const KeyValue_t _key_value_map[4][14] = {
    // Row 0
    {{"`",     KEY_GRAVE,      "~",     KEY_GRAVE,      "esc",   KEY_ESC,       0              },
     {"1",     KEY_1,          "!",     KEY_1,          nullptr, KEY_NONE,      0              },
     {"2",     KEY_2,          "@",     KEY_2,          nullptr, KEY_NONE,      0              },
     {"3",     KEY_3,          "#",     KEY_3,          nullptr, KEY_NONE,      0              },
     {"4",     KEY_4,          "$",     KEY_4,          nullptr, KEY_NONE,      0              },
     {"5",     KEY_5,          "%",     KEY_5,          nullptr, KEY_NONE,      0              },
     {"6",     KEY_6,          "^",     KEY_6,          nullptr, KEY_NONE,      0              },
     {"7",     KEY_7,          "&",     KEY_7,          nullptr, KEY_NONE,      0              },
     {"8",     KEY_8,          "*",     KEY_8,          nullptr, KEY_NONE,      0              },
     {"9",     KEY_9,          "(",     KEY_9,          nullptr, KEY_NONE,      0              },
     {"0",     KEY_0,          ")",     KEY_0,          nullptr, KEY_NONE,      0              },
     {"-",     KEY_MINUS,      "_",     KEY_MINUS,      nullptr, KEY_NONE,      0              },
     {"=",     KEY_EQUAL,      "+",     KEY_EQUAL,      nullptr, KEY_NONE,      0              },
     {"del",   KEY_BACKSPACE,  "del",   KEY_BACKSPACE,  "del",   KEY_DELETE,    0              }},
    // Row 1
    {{"tab",   KEY_TAB,        "tab",   KEY_TAB,        nullptr, KEY_NONE,      0              },
     {"q",     KEY_Q,          "Q",     KEY_Q,          "Q",     KEY_Q,         KEY_MOD_LSHIFT },
     {"w",     KEY_W,          "W",     KEY_W,          "W",     KEY_W,         KEY_MOD_LSHIFT },
     {"e",     KEY_E,          "E",     KEY_E,          "E",     KEY_E,         KEY_MOD_LSHIFT },
     {"r",     KEY_R,          "R",     KEY_R,          "R",     KEY_R,         KEY_MOD_LSHIFT },
     {"t",     KEY_T,          "T",     KEY_T,          "T",     KEY_T,         KEY_MOD_LSHIFT },
     {"y",     KEY_Y,          "Y",     KEY_Y,          "Y",     KEY_Y,         KEY_MOD_LSHIFT },
     {"u",     KEY_U,          "U",     KEY_U,          "U",     KEY_U,         KEY_MOD_LSHIFT },
     {"i",     KEY_I,          "I",     KEY_I,          "I",     KEY_I,         KEY_MOD_LSHIFT },
     {"o",     KEY_O,          "O",     KEY_O,          "O",     KEY_O,         KEY_MOD_LSHIFT },
     {"p",     KEY_P,          "P",     KEY_P,          "P",     KEY_P,         KEY_MOD_LSHIFT },
     {"[",     KEY_LEFTBRACE,  "{",     KEY_LEFTBRACE,  nullptr, KEY_NONE,      0              },
     {"]",     KEY_RIGHTBRACE, "}",     KEY_RIGHTBRACE, nullptr, KEY_NONE,      0              },
     {"\\",    KEY_BACKSLASH,  "|",     KEY_BACKSLASH,  nullptr, KEY_NONE,      0              }},
    // Row 2
    {{"fn",    KEY_NONE,       "fn",    KEY_NONE,       nullptr, KEY_NONE,      0              },
     {"shift", KEY_LEFTSHIFT,  "shift", KEY_LEFTSHIFT,  nullptr, KEY_NONE,      0              },
     {"a",     KEY_A,          "A",     KEY_A,          "A",     KEY_A,         KEY_MOD_LSHIFT },
     {"s",     KEY_S,          "S",     KEY_S,          "S",     KEY_S,         KEY_MOD_LSHIFT },
     {"d",     KEY_D,          "D",     KEY_D,          "D",     KEY_D,         KEY_MOD_LSHIFT },
     {"f",     KEY_F,          "F",     KEY_F,          "F",     KEY_F,         KEY_MOD_LSHIFT },
     {"g",     KEY_G,          "G",     KEY_G,          "G",     KEY_G,         KEY_MOD_LSHIFT },
     {"h",     KEY_H,          "H",     KEY_H,          "H",     KEY_H,         KEY_MOD_LSHIFT },
     {"j",     KEY_J,          "J",     KEY_J,          "J",     KEY_J,         KEY_MOD_LSHIFT },
     {"k",     KEY_K,          "K",     KEY_K,          "K",     KEY_K,         KEY_MOD_LSHIFT },
     {"l",     KEY_L,          "L",     KEY_L,          "L",     KEY_L,         KEY_MOD_LSHIFT },
     {";",     KEY_SEMICOLON,  ":",     KEY_SEMICOLON,  "up",    KEY_UP,        0              },
     {"'",     KEY_APOSTROPHE, "\"",    KEY_APOSTROPHE, nullptr, KEY_NONE,      0              },
     {"enter", KEY_ENTER,      "enter", KEY_ENTER,      nullptr, KEY_NONE,      0              }},
    // Row 3
    {{"ctrl",  KEY_LEFTCTRL,   "ctrl",  KEY_LEFTCTRL,   nullptr, KEY_NONE,      0              },
     {"opt",   KEY_LEFTMETA,   "opt",   KEY_LEFTMETA,   nullptr, KEY_NONE,      0              },
     {"alt",   KEY_LEFTALT,    "alt",   KEY_LEFTALT,    nullptr, KEY_NONE,      0              },
     {"z",     KEY_Z,          "Z",     KEY_Z,          "Z",     KEY_Z,         KEY_MOD_LSHIFT },
     {"x",     KEY_X,          "X",     KEY_X,          "X",     KEY_X,         KEY_MOD_LSHIFT },
     {"c",     KEY_C,          "C",     KEY_C,          "C",     KEY_C,         KEY_MOD_LSHIFT },
     {"v",     KEY_V,          "V",     KEY_V,          "V",     KEY_V,         KEY_MOD_LSHIFT },
     {"b",     KEY_B,          "B",     KEY_B,          "B",     KEY_B,         KEY_MOD_LSHIFT },
     {"n",     KEY_N,          "N",     KEY_N,          "N",     KEY_N,         KEY_MOD_LSHIFT },
     {"m",     KEY_M,          "M",     KEY_M,          "M",     KEY_M,         KEY_MOD_LSHIFT },
     {",",     KEY_COMMA,      "<",     KEY_COMMA,      "left",  KEY_LEFT,      0              },
     {".",     KEY_DOT,        ">",     KEY_DOT,        "down",  KEY_DOWN,      0              },
     {"/",     KEY_SLASH,      "?",     KEY_SLASH,      "right", KEY_RIGHT,     0              },
     {" ",     KEY_SPACE,      " ",     KEY_SPACE,      nullptr, KEY_NONE,      0              }}};
// clang-format on

Keyboard::KeyEvent_t Keyboard::convertToKeyEvent(const KeyEventRaw_t& key)
{
    KeyEvent_t ret;
    ret.state = key.state;

    // Fn key itself - modifier, no keycode
    if (key.row == 2 && key.col == 0) {
        ret.keyCode    = KEY_NONE;
        ret.keyName    = "fn";
        ret.isModifier = true;
        return ret;
    }

    // Fn layer: override selected keys when Fn is held.
    // Keys with no Fn entry (fnKeyCode == KEY_NONE) fall through to the normal
    // lookup below, so they behave as if Fn were not held.  To add a new Fn
    // mapping, only the table needs updating -- no code change required.
    if (_fn_state) {
        const auto& kv = _key_value_map[key.row][key.col];
        if (kv.fnKeyCode != KEY_NONE) {
            ret.keyCode        = kv.fnKeyCode;
            ret.keyName        = kv.fnName;
            ret.isModifier     = false;
            ret.extraModifiers = kv.fnExtraModifiers;
            return ret;
        }
    }

    // Normal key lookup - shift determines upper/symbol layer
    bool use_shifted_version = (modifiers & KEY_MOD_LSHIFT);

    if (use_shifted_version) {
        ret.keyCode = _key_value_map[key.row][key.col].secondKeyCode;
        ret.keyName = _key_value_map[key.row][key.col].secondName;
    } else {
        ret.keyCode = _key_value_map[key.row][key.col].firstKeyCode;
        ret.keyName = _key_value_map[key.row][key.col].firstName;
    }

    ret.isModifier = (ret.keyCode == KEY_LEFTSHIFT || ret.keyCode == KEY_LEFTCTRL ||
                      ret.keyCode == KEY_LEFTALT   || ret.keyCode == KEY_LEFTMETA);

    return ret;
}

