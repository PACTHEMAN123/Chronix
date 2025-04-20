//! char devices

use alloc::{boxed::Box, sync::Arc};
use fdt::{node::{self, FdtNode}, Fdt};
use hal::constant::{Constant, ConstantsHal};

use crate::drivers::serial::{uart::Uart, Serial};

/// scan the device tree and return a Serial
pub fn scan_char_device(device_tree: &Fdt) -> Arc<Serial> {
    let chosen = device_tree.chosen();
    let mut stdout = chosen.stdout();
    if stdout.is_none() {
        log::info!("[device tree]: no standard stdout device, trying another");
        let chosen = device_tree.find_node("/chosen").expect("No chosen node!");
        let stdout_path = chosen
            .properties()
            .find(|n| n.name == "stdout-path")
            .and_then(|n| {
                let bytes = unsafe {
                    core::slice::from_raw_parts_mut((n.value.as_ptr()) as *mut u8, n.value.len())
                };
                let mut len = 0;
                for byte in bytes.iter() {
                    if *byte == b':' {
                        return core::str::from_utf8(&n.value[..len]).ok();
                    }
                    len += 1;
                }
                core::str::from_utf8(&n.value[..n.value.len() - 1]).ok()
            })
            .unwrap();
        log::info!("[device tree]: searching stdout: {}", stdout_path);
        stdout = device_tree.find_node(stdout_path);
    }
    if stdout.is_none() {
        log::info!("Unable to parse /chosen, choosing first serial device");
        stdout = device_tree.find_compatible(&[
            "ns16550a",
            "snps,dw-apb-uart", // C910, VF2
        ])
    }
    let stdout = stdout.expect("failed to get stdout device");
    Arc::new(get_serial(&stdout))
}

/// use the given the device tree node
/// treat it as serial and return a Seraial Instance
pub fn get_serial(stdout: &FdtNode) -> Serial {
    let reg = stdout.reg().unwrap().next().unwrap();
    let base_paddr = reg.starting_address as usize;
    let size = reg.size.unwrap();
    let base_vaddr = base_paddr | Constant::KERNEL_ADDR_SPACE.start;
    let irq_number = stdout.property("interrupts").unwrap().as_usize().unwrap();
    log::info!("[device tree]: Serial IRQ number: {}", irq_number);
    let first_compatible = stdout.compatible().unwrap().first();
    match first_compatible {
        "ns16550a" | "snps,dw-apb-uart" => {
            // Parse clock frequency
            let freq_raw = stdout
                .property("clock-frequency")
                .expect("No clock-frequency property of stdout serial device")
                .as_usize()
                .expect("Parse clock-frequency to usize failed");
            let mut reg_io_width = 1;
            if let Some(reg_io_width_raw) = stdout.property("reg-io-width") {
                reg_io_width = reg_io_width_raw
                    .as_usize()
                    .expect("Parse reg-io-width to usize failed");
            }
            let mut reg_shift = 0;
            if let Some(reg_shift_raw) = stdout.property("reg-shift") {
                reg_shift = reg_shift_raw
                    .as_usize()
                    .expect("Parse reg-shift to usize failed");
            }
            log::info!("uart: base_paddr:{base_paddr:#x}, size:{size:#x}, reg_io_width:{reg_io_width}, reg_shift:{reg_shift}");

            let uart = unsafe {
                Uart::new(
                    base_vaddr,
                    freq_raw,
                    115200,
                    reg_io_width,
                    reg_shift,
                    first_compatible == "snps,dw-apb-uart",
                )
            };
            Serial::new(base_paddr, size, irq_number, Box::new(uart))
        }
        _ => panic!("Unsupported serial console"),
    }
}