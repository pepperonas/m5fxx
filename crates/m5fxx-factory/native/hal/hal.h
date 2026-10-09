#pragma once
#include <M5Unified.hpp>
#include "keyboard.h"
#include <string>
#include <map>
#include <vector>
#include <utility>
#include <TinyGPS++.h>
struct Settings {std::map<std::string,std::string> strings;std::map<std::string,bool> flags;std::string GetString(const std::string& k,const std::string& d=""){return strings.count(k)?strings[k]:d;}void SetString(const std::string& k,const std::string& v){strings[k]=v;}bool GetBool(const std::string& k,bool d=false){return flags.count(k)?flags[k]:d;}void SetBool(const std::string& k,bool v){flags[k]=v;}};
class CapLoRa868 {public:bool available=false;TinyGPSPlus gps;struct lora_config{static constexpr float ferq=868,bw=500;static constexpr int sf=7,cr=5,syncWord=0x34,power=10,preambleLength=10;};mclog::Signal<const std::string&> onLoraMsg;bool init(){return available;}bool loraSendMsg(const std::string& msg){if(available)onLoraMsg.emit(msg);return available;}TinyGPSPlus* borrowGPS(){return available?&gps:nullptr;}void returnGPS(){} };
class Hal {public:
M5GFX& display=M5.Display;LGFX_Sprite canvas{&M5.Display},canvasSystemBar{&M5.Display},canvasKeyboardBar{&M5.Display};
m5::Speaker_Class& speaker=M5.Speaker;m5::Mic_Class& mic=M5.Mic;m5::Button_Class& homeButton=M5.BtnA;m5::IMU_Class& imu=M5.Imu;Keyboard keyboard;CapLoRa868 capLora868;Settings settings;
uint8_t battery=100;bool wifi=false,ble=false,usb=false,sd=true;std::string received;
void init();void update();void feedTheDog();void delay(uint32_t ms){lgfx::delay(ms);}uint32_t millis(){return lgfx::millis();}
void pushCanvas(){canvas.pushSprite(36,26);publish();}void pushCanvasKeyboardBar(){canvasKeyboardBar.pushSprite(0,0);publish();}void pushCanvasSystemBar(){canvasSystemBar.pushSprite(36,0);publish();}void publish();
std::vector<uint8_t> getDeviceMac(){return {2,0,0,0,0,1};}std::string getDeviceMacString(){return "02:00:00:00:00:01";}uint8_t getBatLevel(){return battery;}
using ScanResult_t=std::pair<int,std::string>;void wifiInit(){}void wifiDeinit(){}void wifiScan(std::vector<ScanResult_t>& out){out={{-45,"Simulated ADV"},{-72,"Test Network"}};}bool wifiConnect(const std::string&,const std::string&){wifi=true;return true;}bool isWifiConnected()const{return wifi;}void wifiDisconnect(){wifi=false;}bool isTimeSynced()const{return false;}
void espNowInit(){}void espNowDeinit(){}void espNowSend(const std::string& s){received=s;}bool espNowAvailable(){return !received.empty();}const std::string& espNowGetReceivedData(){return received;}void espNowClearReceivedData(){received.clear();}
void irInit(){}void irSend(uint8_t,uint8_t){}void bleKeyboardInit(){}bool bleKeyboardIsConnected()const{return ble;}void usbKeyboardInit(){}bool usbKeyboardIsConnected()const{return usb;}Settings& getSettings(){return settings;}
struct SdCardProbeResult_t {bool is_mounted=false;std::string size,type,name;bool operator==(const SdCardProbeResult_t& o)const{return is_mounted==o.is_mounted&&size==o.size&&type==o.type&&name==o.name;}bool operator!=(const SdCardProbeResult_t& o)const{return !(*this==o);}};SdCardProbeResult_t sdCardProbe(){return {sd,"128 MB","SDHC","virtual_sd"};}
};
Hal& GetHAL();
