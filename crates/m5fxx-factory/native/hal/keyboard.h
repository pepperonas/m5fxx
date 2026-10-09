#pragma once
#include <cstdint>
#include <deque>
#include <mooncake_log_signal.h>
#include <keymap.h>
class Keyboard {public:
struct KeyEventRaw_t{bool state=false;uint8_t row=0,col=0;};struct KeyEvent_t{bool state=false,isModifier=false;KeScanCode_t keyCode=KEY_NONE;const char* keyName="";uint8_t extraModifiers=0;};
mclog::Signal<const KeyEventRaw_t&> onKeyEventRaw;mclog::Signal<const KeyEvent_t&> onKeyEvent;
KeyEventRaw_t raw;KeyEvent_t event;uint8_t modifiers=0;bool _fn_state=false;KeyEvent_t convertToKeyEvent(const KeyEventRaw_t& key);uint8_t getModifierMask(){return modifiers;}const KeyEvent_t& getLatestKeyEvent(){return event;}const KeyEventRaw_t& getLatestKeyEventRaw(){return raw;}void clearKeyEvent(){raw={};event={};}void update();bool init(){return true;}
};
