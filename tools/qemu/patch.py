#!/usr/bin/env python3
"""Apply the Cardputer bridge to pinned Espressif QEMU source (GPL-2.0+)."""
from pathlib import Path
import sys
root = Path(sys.argv[1])

def edit(relative, old, new, count=1):
    path=root/relative
    text=path.read_text()
    if text.count(old)!=count:
        raise SystemExit(f"Unexpected QEMU source in {relative}: expected {count} matches; use the pinned commit")
    path.write_text(text.replace(old,new))

edit('meson.build', "  slirp = declare_dependency(dependencies: [slirp_dep],\n                          compile_args: slirp_cflags,\n                          version: slirp_dep.version())", "  if slirp_dep.found()\n    slirp = declare_dependency(dependencies: [slirp_dep],\n                              compile_args: slirp_cflags,\n                              version: slirp_dep.version())\n  endif")
# Avoid comparing the transferred byte with a buffer length.
edit('hw/ssi/esp32s3_spi.c','if (byte < tx_bytes)','if (i < tx_bytes)')
edit('hw/ssi/esp32s3_spi.c','if (byte < rx_bytes)','if (i < rx_bytes)')
# The byte-oriented SSI flash model does not model parallel data lanes.
edit('hw/ssi/esp32s3_spi.c','    /* Check which CS line is active (low) */', '''    /* Normalize parallel flash-read lanes to the byte-oriented SSI bus. */
    if (t->cmd == 0xeb || t->cmd == 0xbb || t->cmd == 0x6b || t->cmd == 0x3b) {
        if (t->addr_bytes == 4) t->addr >>= 8;
        t->addr_bytes = 3;
        t->cmd = CMD_READ;
        t->dummy_bytes = 0;
    }
    /* Check which CS line is active (low) */''')
# GD25Q64 uses SR2's QE bit with commands 31h / 35h, like Winbond.
p=root/'hw/block/m25p80.c';text=p.read_text()
for start,end in [('    case WRSR2:', '    case BRWR:'), ('    case WRSR2:', '    case WRDI:'), ('    case RDCR_EQIO:', '        default:')]:
    offset=0 if end=='    case BRWR:' else text.index('static void decode_new_cmd')
    a=text.index(start,offset);b=text.index(end,a)
    section=text[a:b]
    if section.count('case MAN_WINBOND:')!=1:raise SystemExit('Unexpected flash source')
    text=text[:a]+section.replace('case MAN_WINBOND:', 'case MAN_WINBOND:\n        case MAN_GIGADEVICE:')+text[b:]
p.write_text(text)
edit('hw/xtensa/esp32s3.c','static void esp32s3_machine_init(MachineState *machine)', '#include "m5fxx_cardputer.c"\n\nstatic void esp32s3_machine_init(MachineState *machine)')
edit('hw/xtensa/esp32s3.c','    /* GPIO realization */','    m5_init(ss, sys_mem);\n\n    /* GPIO realization */')
(root/'hw/xtensa/m5fxx_cardputer.c').write_bytes(Path(__file__).with_name('m5fxx_cardputer.c').read_bytes())
