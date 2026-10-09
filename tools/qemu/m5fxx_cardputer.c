/* SPDX-License-Identifier: GPL-2.0-or-later
 * Cardputer board bridge for Espressif QEMU 9.2.2.
 * Included by hw/xtensa/esp32s3.c. No host radio/network forwarding.
 */
#include "chardev/char-fe.h"
#include "sysemu/runstate.h"
#include "qemu/timer.h"

typedef struct M5Cardputer M5Cardputer;
typedef struct M5Region {
    MemoryRegion mr;
    M5Cardputer *board;
    unsigned kind;
    uint32_t regs[1024];
} M5Region;
struct M5Cardputer {
    Esp32s3SocState *soc;
    CharBackend bridge;
    M5Region region[8];
    uint64_t output;
    bool keys[4][14], home;
    uint8_t incoming[4];
    unsigned incoming_len;
    int64_t pressed_at[4][14], release_at[4][14];
    QEMUTimer *key_timer;
    uint8_t lcd_command;
};

static void m5_send(M5Cardputer *b, uint8_t kind, const uint8_t *data, unsigned len)
{
    uint8_t header[5] = {kind, len, len >> 8, len >> 16, len >> 24};
    qemu_chr_fe_write_all(&b->bridge, header, sizeof(header));
    if (len) qemu_chr_fe_write_all(&b->bridge, data, len);
}
static void m5_spi(M5Region *r)
{
    M5Cardputer *b = r->board;
    uint32_t *regs = r->regs;
    unsigned len = ((regs[0x1c/4] & 0x3ffff) + 8) / 8;
    unsigned user = regs[0x10/4];
    bool dc = (b->output >> 34) & 1;
    bool selected = !((b->output >> 37) & 1);
    if (len > 32768) len = 32768;
    uint8_t *data = g_malloc0(MAX(len, 64));
    if (regs[0x30/4] & (1U << 28)) {
        uint32_t channel;
        if (esp_gdma_get_channel_periph(&b->soc->gdma.parent,
                r->kind == 1 ? GDMA_SPI2 : GDMA_SPI3, ESP_GDMA_OUT_IDX, &channel)) {
            esp_gdma_read_channel(&b->soc->gdma.parent, channel, data, len);
        }
    } else {
        memcpy(data, &regs[(user & (1U<<25)) ? 0xb8/4 : 0x98/4], MIN(len,64));
    }
    if (selected && (user & (1U << 27))) {
        m5_send(b, dc ? 1 : 0, data, len);
        if (!dc && len) b->lcd_command = data[len-1];
    }
    if (user & (1U << 28)) {
        unsigned offset = (user & (1U<<24)) ? 0xb8/4 : 0x98/4;
        memset(&regs[offset], 0, 32);
        // ST7789 identification and display status, returned in read-data order.
        if (b->lcd_command == 4) regs[offset] = 0x00858585;
        else if (b->lcd_command == 9) regs[offset] = 0x00e085;
    }
    g_free(data);
    regs[0] &= ~((1U<<24) | (1U<<23));
    regs[0x3c/4] |= 1U<<12;
    qemu_set_irq(qdev_get_gpio_in(DEVICE(&b->soc->intmatrix),
            r->kind == 1 ? ETS_SPI2_INTR_SOURCE : ETS_SPI3_INTR_SOURCE),
            !!(regs[0x34/4] & regs[0x3c/4]));
}

static uint64_t m5_read(void *opaque, hwaddr addr, unsigned size)
{
    M5Region *r = opaque;
    M5Cardputer *b = r->board;
    if (r->kind == 0) {
        if (addr == 0x38) return b->soc->gpio.parent.strap_mode;
        if (addr == 0x3c || addr == 0x40) {
            uint64_t in = b->output;
            // Pull-up keyboard inputs and G0. Respect IO_MUX pull-down probes.
            const unsigned inputs[] = {0,13,15,3,4,5,6,7};
            for (unsigned j=0; j<ARRAY_SIZE(inputs); ++j) {
                unsigned pin = inputs[j];
                if (!(b->region[4].regs[pin+1] & (1U<<7))) in |= 1ULL<<pin;
            }
            if (b->home) in &= ~1ULL;
            unsigned scan = ((b->output>>8)&1) | (((b->output>>9)&1)<<1) | (((b->output>>11)&1)<<2);
            const unsigned pins[] = {13,15,3,4,5,6,7};
            unsigned row = 3-(scan&3);
            for(unsigned j=0;j<7;++j) {
                unsigned col = 2*j + (scan<4);
                if (b->keys[row][col]) in &= ~(1ULL<<pins[j]);
            }
            return addr == 0x3c ? (uint32_t)in : in>>32;
        }
        if (addr == 4) return (uint32_t)b->output;
        if (addr == 0x10) return b->output>>32;
    }
    // Synthetic battery ADC: conversions complete with a stable sample.
    if(r->kind==7 && (addr==12 || addr==48)) return (1U<<16)|(1U<<17)|3000;
    // SPI interrupt status follows raw & enable.
    if ((r->kind==1 || r->kind==2) && addr==0x40)
        return r->regs[0x3c/4] & r->regs[0x34/4];
    return r->regs[addr/4];
}
static void m5_write(void *opaque, hwaddr addr, uint64_t value, unsigned size)
{
    M5Region *r = opaque;
    M5Cardputer *b = r->board;
    r->regs[addr/4] = value;
    if (r->kind==0) {
        switch(addr) {
        case 4: b->output = (b->output & 0xffffffff00000000ULL) | (uint32_t)value; break;
        case 8: b->output |= (uint32_t)value; break;
        case 12: b->output &= ~(uint64_t)(uint32_t)value; break;
        case 0x10: b->output = (b->output & 0xffffffffULL) | (value<<32); break;
        case 0x14: b->output |= value<<32; break;
        case 0x18: b->output &= ~(value<<32); break;
        }
    } else if(r->kind==3) {
        unsigned signal=b->region[0].regs[(0x554+38*4)/4]&0x1ff;
        if(signal>=73 && signal<81) {
            unsigned channel=signal-73;
            unsigned timer=r->regs[channel*5]&3;
            unsigned resolution=r->regs[(0xa0+8*timer)/4]&15;
            unsigned duty=(r->regs[channel*5+2]>>4)&0x7fff;
            if(resolution) {
                uint8_t brightness=MIN(255,(duty*255)/((1U<<resolution)-1));
                m5_send(b,2,&brightness,1);
            }
            r->regs[channel*5+4]=r->regs[channel*5+2];
        }
    } else if (r->kind==1 || r->kind==2) {
        if (addr==0 && (value & (1U<<24))) m5_spi(r);
        if (addr==0) r->regs[0] &= ~((1U<<24)|(1U<<23));
        if (addr==0x38) {
            r->regs[0x3c/4] &= ~value;
            qemu_set_irq(qdev_get_gpio_in(DEVICE(&b->soc->intmatrix),
                r->kind==1 ? ETS_SPI2_INTR_SOURCE : ETS_SPI3_INTR_SOURCE), 0);
        }
    }
}
static const MemoryRegionOps m5_ops = {
    .read=m5_read, .write=m5_write, .endianness=DEVICE_LITTLE_ENDIAN,
    .valid.min_access_size=4, .valid.max_access_size=4,
};
static void m5_key_tick(void *opaque)
{
    M5Cardputer *b=opaque;
    int64_t now=qemu_clock_get_ns(QEMU_CLOCK_VIRTUAL);
    bool pending=false;
    for(unsigned row=0;row<4;++row) for(unsigned col=0;col<14;++col) {
        if(b->release_at[row][col]) {
            if(now>=b->release_at[row][col]) {b->keys[row][col]=false;b->release_at[row][col]=0;}
            else pending=true;
        }
    }
    if(pending) timer_mod(b->key_timer,now+5*1000*1000);
}
static int m5_can_read(void *opaque) { return 4; }
static void m5_receive(void *opaque, const uint8_t *bytes, int size)
{
    M5Cardputer *b = opaque;
    for(int i=0;i<size;++i) {
        b->incoming[b->incoming_len++] = bytes[i];
        unsigned need = b->incoming[0]=='K' ? 4 : (b->incoming[0]=='H' || b->incoming[0]=='P') ? 2 : 1;
        if(b->incoming_len==need) {
            if(b->incoming[0]=='K' && b->incoming[1]<4 && b->incoming[2]<14) {
                unsigned row=b->incoming[1],col=b->incoming[2];
                int64_t now=qemu_clock_get_ns(QEMU_CLOCK_VIRTUAL);
                if(b->incoming[3]) {b->keys[row][col]=true;b->pressed_at[row][col]=now;b->release_at[row][col]=0;}
                else {
                    b->release_at[row][col]=MAX(now,b->pressed_at[row][col]+50*1000*1000);
                    timer_mod(b->key_timer,now+5*1000*1000);
                }
            }
            else if(b->incoming[0]=='H') b->home=!!b->incoming[1];
            else if(b->incoming[0]=='P') {if(b->incoming[1]) vm_stop(RUN_STATE_PAUSED);else vm_start();}
            else if(b->incoming[0]=='R') qemu_system_reset_request(SHUTDOWN_CAUSE_HOST_QMP_SYSTEM_RESET);
            b->incoming_len=0;
        }
    }
}
static void m5_init(Esp32s3SocState *soc, MemoryRegion *memory)
{
    Chardev *chr = qemu_chr_find("cardputer");
    if(!chr) return;
    M5Cardputer *b = g_new0(M5Cardputer,1);
    b->soc=soc;
    b->key_timer=timer_new_ns(QEMU_CLOCK_VIRTUAL,m5_key_tick,b);
    b->output=(1ULL<<37)|(1ULL<<33);
    qemu_chr_fe_init(&b->bridge,chr,&error_fatal);
    qemu_chr_fe_set_handlers(&b->bridge,m5_can_read,m5_receive,NULL,NULL,b,NULL,true);
    static const hwaddr bases[] = {0x60004000,0x60024000,0x60025000,0x60019000,0x60009000,0x60013000,0x60027000,0x60008800};
    for(unsigned i=0;i<ARRAY_SIZE(bases);++i) {
        M5Region *r=&b->region[i]; r->board=b; r->kind=i;
        memory_region_init_io(&r->mr,OBJECT(soc),&m5_ops,r,"m5fxx-cardputer",i==7 ? 0x400 : 0x1000);
        memory_region_add_subregion_overlap(memory,bases[i],&r->mr,10);
        if(i==1 || i==2) r->regs[0xc/4]=0x80000000;
    }
    m5_send(b,3,(const uint8_t*)"M5FXXCP1",8);
}
