# boot cmd for board

set the host ip

```bash
setenv serverip 10.250.233.77
```
load the kernel image file from tftp server to board

```bash
tftpboot 0x80200000 zImage
```

load the device tree (maybe not using?)

```bash
tftpboot ${fdt_addr_r} jh7110.dtb
```

start to boot

```bash
bootm 0x80200000 - ${fdt_addr_r}
```

# problem and solutions (VisionFive2)

## no output after boot?

Use OpenSBI interface for OS debug info, maybe the uart driver is not correct.

## pagefault when accessing memory?

Maybe we should always open the Access bit and Dirty bit in PTE?

## cannot read from keyboard input?

Check if the plic / externel interrupt is on

## terrible output of user programs using tty

qemu sometimes ignore '\r'. check the terminos and tty
