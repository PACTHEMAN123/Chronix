#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ahci_cmd_hdr {
  uint32_t opts;
  uint32_t status;
  uint32_t tbl_addr_lo;
  uint32_t tbl_addr_hi;
  uint32_t reserved[4];
} ahci_cmd_hdr;

typedef struct ahci_sg {
  uint32_t addr_lo;
  uint32_t addr_hi;
  uint32_t reserved;
  uint32_t flags_size;
} ahci_sg;

typedef struct ahci_ioport {
  uint64_t port_mmio;
  struct ahci_cmd_hdr *cmd_slot;
  uint64_t cmd_slot_dma;
  uint64_t rx_fis;
  uint64_t rx_fis_dma;
  uint64_t cmd_tbl;
  uint64_t cmd_tbl_dma;
  struct ahci_sg *cmd_tbl_sg;
} ahci_ioport;

typedef struct ahci_blk_dev {
  bool lba48;
  uint64_t lba;
  uint64_t blksz;
  uint32_t queue_depth;
  uint8_t product[41];
  uint8_t serial[21];
  uint8_t revision[9];
} ahci_blk_dev;

typedef struct ahci_device {
  uint64_t mmio_base;
  uint32_t flags;
  uint32_t cap;
  uint32_t cap2;
  uint32_t version;
  uint32_t port_map;
  uint32_t pio_mask;
  uint32_t udma_mask;
  uint8_t n_ports;
  uint32_t port_map_linkup;
  struct ahci_ioport port[32];
  uint8_t port_idx;
  struct ahci_blk_dev blk_dev;
} ahci_device;

extern uint64_t ahci_malloc_align(uint64_t size, uint32_t align);

extern void ahci_mdelay(uint32_t ms);

extern uint64_t ahci_phys_to_uncached(uint64_t va);

extern int32_t ahci_printf(const char *fmt, ...);

extern int32_t ahci_init(struct ahci_device *ahci_dev);

extern uint64_t ahci_sata_read_common(const struct ahci_device *ahci_dev,
                                    uint64_t blknr,
                                    uint32_t blkcnt,
                                    void *buffer);

extern uint64_t ahci_sata_write_common(const struct ahci_device *ahci_dev,
                                     uint64_t blknr,
                                     uint32_t blkcnt,
                                     void *buffer);

extern void ahci_sync_dcache(void);

extern uint64_t ahci_virt_to_phys(uint64_t va);
