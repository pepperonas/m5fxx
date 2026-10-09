#include <hal.h>
#include <mooncake.h>
#include <smooth_ui_toolkit.h>
#include <apps/app_launcher/app_launcher.h>
#include <apps/app_wifi_scan/app_wifi_scan.h>
#include <apps/app_record/app_record.h>
#include <apps/app_chat/app_chat.h>
#include <apps/app_remote/app_remote.h>
#include <apps/app_set_wifi/app_set_wifi.h>
#include <apps/app_repl/app_repl.h>
#include <apps/app_clock/app_clock.h>
#include <apps/app_keyboard/app_keyboard.h>
#include <apps/app_imu/app_imu.h>
#include <apps/app_sdcard/app_sdcard.h>
#include <apps/app_stringir_toolkit/app_stringir_toolkit.h>
#include <apps/app_lora_chat/app_lora_chat.h>
#include <apps/app_gps/app_gps.h>
#include <apps/utils/audio/audio.h>
#include <mutex>
#include <condition_variable>
#include <thread>
#include <atomic>
#include <array>
#include <deque>
#include <stdexcept>
Unified M5;
static Hal hal;
Hal& GetHAL(){return hal;}
namespace {
std::mutex gate;std::condition_variable cv;std::thread worker;
uint64_t ticks=0;bool stopping=false,started=false,waiting=false;
std::array<uint16_t,240*135> frame{};uint8_t frame_brightness=255;
std::deque<Keyboard::KeyEventRaw_t> keys;bool g0=false;
struct Stop {};
}
namespace lgfx {inline namespace v1 {
unsigned long millis(){std::lock_guard<std::mutex> lock(gate);return ticks;}unsigned long micros(){return millis()*1000;}
void delay(unsigned long ms){std::unique_lock<std::mutex> lock(gate);auto target=ticks+ms;waiting=true;cv.notify_all();cv.wait(lock,[&]{return stopping||ticks>=target;});waiting=false;if(stopping)throw Stop{};}
void delayMicroseconds(unsigned int us){delay((us+999)/1000);}
}}
void Hal::publish(){std::lock_guard<std::mutex> lock(gate);for(int y=0;y<135;y++)for(int x=0;x<240;x++)frame[y*240+x]=display.readPixelValue(x,y);frame_brightness=display.brightness;}
void Hal::feedTheDog(){publish();delay(1);}
void Hal::init(){display.begin();canvas.setColorDepth(16);canvasSystemBar.setColorDepth(16);canvasKeyboardBar.setColorDepth(16);canvas.createSprite(204,109);canvasSystemBar.createSprite(204,26);canvasKeyboardBar.createSprite(36,135);canvas.setSwapBytes(true);canvasSystemBar.setSwapBytes(true);canvasKeyboardBar.setSwapBytes(true);}
void Hal::update(){homeButton.previous=homeButton.pressed;{std::lock_guard<std::mutex> lock(gate);homeButton.pressed=g0;}keyboard.update();}
void Keyboard::update(){clearKeyEvent();{std::lock_guard<std::mutex> lock(gate);if(keys.empty())return;raw=keys.front();keys.pop_front();}
if(raw.row==2&&raw.col==0)_fn_state=raw.state;
event=convertToKeyEvent(raw);
uint8_t bit=event.keyCode==KEY_LEFTSHIFT?2:event.keyCode==KEY_LEFTCTRL?1:event.keyCode==KEY_LEFTALT?4:event.keyCode==KEY_LEFTMETA?8:0;
if(raw.state)modifiers|=bit;else modifiers&=~bit;onKeyEventRaw.emit(raw);onKeyEvent.emit(event);
}
namespace audio {
static bool quiet=false;void set_keyboard_sfx_enable(bool){}bool is_quiet_mode(){return quiet;}void set_quiet_mode(bool q){quiet=q;}void play_random_tone(int,double){}void play_tone(int,double){}void play_melody(const std::vector<int>&,double){}void play_tone_from_midi(int,double){}void play_next_tone(){}void play_keyboard_tone(){}
}
extern "C" void m5fxx_factory_init(){if(started)return;started=true;stopping=false;ticks=0;worker=std::thread([]{try{hal.init();smooth_ui_toolkit::ui_hal::on_delay([](uint32_t ms){hal.delay(ms);});smooth_ui_toolkit::ui_hal::on_get_tick([]{return hal.millis();});auto& m=mooncake::GetMooncake();m.installApp(std::make_unique<Launcher>());m.installApp(std::make_unique<AppWifiScan>());m.installApp(std::make_unique<AppRecord>());m.installApp(std::make_unique<AppChat>());m.installApp(std::make_unique<AppRemote>());m.installApp(std::make_unique<AppREPL>());m.installApp(std::make_unique<AppSetWiFi>());m.installApp(std::make_unique<AppClock>());m.installApp(std::make_unique<AppKeyboard>());m.installApp(std::make_unique<AppImu>());m.installApp(std::make_unique<AppSdcard>());m.installApp(std::make_unique<AppStringIRToolKit>());m.installApp(std::make_unique<AppLoraChat>());m.installApp(std::make_unique<AppGPS>());while(true){hal.feedTheDog();hal.update();m.update();}}catch(const Stop&){} });}
extern "C" void m5fxx_factory_step(uint32_t ms,uint16_t* out,uint8_t* brightness){std::unique_lock<std::mutex> lock(gate);ticks+=ms;cv.notify_all();cv.wait_for(lock,std::chrono::milliseconds(30),[]{return waiting;});std::copy(frame.begin(),frame.end(),out);*brightness=frame_brightness;}
extern "C" void m5fxx_factory_key(uint8_t row,uint8_t col,bool pressed){if(row>=4||col>=14)return;std::lock_guard<std::mutex> lock(gate);keys.push_back({pressed,row,col});}
extern "C" void m5fxx_factory_home(bool pressed){std::lock_guard<std::mutex> lock(gate);g0=pressed;}
extern "C" void m5fxx_factory_shutdown(){if(!started)return;{std::lock_guard<std::mutex> lock(gate);stopping=true;cv.notify_all();}worker.join();started=false;}
