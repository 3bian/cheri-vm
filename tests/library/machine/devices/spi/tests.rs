// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::machine::devices::Device;
use crate::machine::devices::spi::registers::Register;
use crate::machine::devices::spi::*;

#[test]
fn a_selected_peripheral_echoes_and_no_selection_is_an_error() {
    let mut spi = Spi::new();
    spi.attach(2, Box::new(LoopbackPeripheral)).unwrap();
    spi.write(Register::ChipSelect.offset(), 4, 2).unwrap();
    spi.write(Register::Data.offset(), 4, 0x95).unwrap();
    assert_eq!(spi.read(Register::Data.offset(), 4).unwrap(), 0x95);
    spi.write(Register::ChipSelect.offset(), 4, 0).unwrap();
    spi.write(Register::Data.offset(), 4, 42).unwrap();
    assert_eq!(spi.read(Register::Status.offset(), 4).unwrap(), 3);
}

#[test]
fn host_spi_orders_selection_and_transfers_and_latches_failures() {
    use std::cell::RefCell;
    use std::io;
    use std::rc::Rc;

    struct Peripheral(Rc<RefCell<Vec<String>>>);
    impl SpiPeripheral for Peripheral {
        fn select(
            &mut self,
            selected: bool,
        ) -> crate::machine::error::HostResult<()> {
            self.0.borrow_mut().push(format!("select {selected}"));
            Ok(())
        }
        fn transfer(
            &mut self,
            value: u8,
            mode: u8,
        ) -> crate::machine::error::HostResult<u8> {
            self.0.borrow_mut().push(format!("transfer {value} {mode}"));
            if value == 0xff {
                return Err(io::Error::other("injected SPI failure").into());
            }
            Ok(0x5a)
        }
    }
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut spi = Spi::new();
    spi.replace_peripheral(0, Box::new(Peripheral(calls.clone())))
        .unwrap();
    assert!(spi.write(0, 1, 7).is_err());
    assert!(spi.write(Register::ChipSelect.offset(), 4, 4).is_err());
    assert!(calls.borrow().is_empty());
    spi.write(Register::Mode.offset(), 4, 3).unwrap();
    spi.write(Register::ChipSelect.offset(), 4, 0).unwrap();
    spi.write(Register::ChipSelect.offset(), 4, 0).unwrap();
    spi.write(Register::Data.offset(), 4, 0x42).unwrap();
    assert_eq!(spi.read(Register::Data.offset(), 4).unwrap(), 0x5a);
    spi.write(Register::ChipSelect.offset(), 4, u32::MAX)
        .unwrap();
    assert_eq!(
        *calls.borrow(),
        ["select true", "transfer 66 3", "select false"]
    );
    spi.write(Register::ChipSelect.offset(), 4, 0).unwrap();
    assert!(spi.write(Register::Data.offset(), 4, 0xff).is_err());
    let count = calls.borrow().len();
    assert!(spi.host_error().unwrap().contains("injected SPI failure"));
    assert!(spi.write(Register::Data.offset(), 4, 0x42).is_err());
    assert_eq!(calls.borrow().len(), count);
}

#[test]
fn spi_completion_ack_and_invalid_selection_are_atomic() {
    let mut s = Spi::new();
    s.attach(0, Box::new(LoopbackPeripheral)).unwrap();
    s.write(Register::ChipSelect.offset(), 4, 0).unwrap();
    s.write(Register::Control.offset(), 4, 1).unwrap();

    for mode in 0..4 {
        s.write(Register::Mode.offset(), 4, mode).unwrap();
        s.write(Register::Data.offset(), 4, 0xa5).unwrap();
        assert_eq!(s.read(Register::Data.offset(), 4).unwrap(), 0xa5);
    }

    assert!(s.interrupt().is_some());
    s.write(Register::Acknowledge.offset(), 4, 1).unwrap();
    assert!(s.interrupt().is_none());
    assert!(s.write(Register::ChipSelect.offset(), 4, 4).is_err());
    assert_eq!(s.read(Register::ChipSelect.offset(), 4).unwrap(), 0);
}
