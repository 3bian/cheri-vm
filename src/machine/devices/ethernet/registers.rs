// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

registers! {
    Register {
        Link = 0, "link";
        Status = 4, "status";
        TransmitLength = 8, "transmitLength";
        TransmitData = 12, "transmitData";
        TransmitSend = 16, "transmitSend";
        ReceiveLength = 20, "receiveLength";
        ReceiveData = 24, "receiveData";
        ReceivePop = 28, "receivePop";
        InterruptMask = 32, "interruptMask";
    }
}

pub(crate) const RECEIVE_POP: u32 = 1 << 0;

pub(crate) const STATUS_ALL: u32 =
    STATUS_TRANSMIT_COMPLETE | STATUS_RECEIVE_READY | STATUS_TRANSFER_ERROR;

pub(crate) const STATUS_RECEIVE_READY: u32 = 1 << 1;
pub(crate) const STATUS_TRANSFER_ERROR: u32 = 1 << 2;
pub(crate) const STATUS_TRANSMIT_COMPLETE: u32 = 1 << 0;
pub(crate) const TRANSMIT_SEND: u32 = 1;
