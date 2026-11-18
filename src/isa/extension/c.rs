// Copyright 3bian Limited and Contributors
// SPDX-License-Identifier: MIT

use crate::isa::decode::sign_extend;
use crate::isa::instruction::{
    BranchCondition, ImmediateOperation, Instruction, LoadKind,
    RegisterOperation, StoreKind,
};

fn bits(value: u16, shift: u32, mask: u16) -> u32 {
    ((value >> shift) & mask) as u32
}

fn branch_immediate(value: u16) -> i32 {
    let result = (bits(value, 12, 1) << 8)
        | (bits(value, 10, 3) << 3)
        | (bits(value, 5, 3) << 6)
        | (bits(value, 3, 3) << 1)
        | (bits(value, 2, 1) << 5);
    sign_extend(result, 9)
}

#[allow(clippy::too_many_lines)]
pub fn compressed(value: u16, xlen: u32) -> Option<Instruction> {
    if value & 3 == 3 {
        return None;
    }
    if xlen == 64
        && let Some(decided) = doubleword(value)
    {
        return decided;
    }

    let quadrant = value & 3;
    let function_code = bits(value, 13, 7);
    use Instruction::*;
    let instruction = if is_hint(value) {
        Hint
    } else {
        match (quadrant, function_code) {
            (0, 0) => {
                let destination = prime_register(value, 2);
                let immediate = (bits(value, 7, 15) << 6)
                    | (bits(value, 11, 3) << 4)
                    | (bits(value, 5, 1) << 3)
                    | (bits(value, 6, 1) << 2);

                if immediate == 0 {
                    return None;
                }

                Immediate {
                    operation: ImmediateOperation::Add,
                    destination,
                    source: 2,
                    immediate: immediate as i32,
                }
            }
            (0, 2) => {
                let offset = (bits(value, 10, 7) << 3)
                    | (bits(value, 6, 1) << 2)
                    | (bits(value, 5, 1) << 6);
                Load {
                    kind: LoadKind::Word,
                    destination: prime_register(value, 2),
                    base: prime_register(value, 7),
                    offset: offset as i32,
                }
            }
            (0, 6) => {
                let offset = (bits(value, 10, 7) << 3)
                    | (bits(value, 6, 1) << 2)
                    | (bits(value, 5, 1) << 6);
                Store {
                    kind: StoreKind::Word,
                    source: prime_register(value, 2),
                    base: prime_register(value, 7),
                    offset: offset as i32,
                }
            }
            (1, 0) => {
                let destination = unsigned_register(value, 7);
                Immediate {
                    operation: ImmediateOperation::Add,
                    destination,
                    source: destination,
                    immediate: compressed_immediate(value),
                }
            }
            (1, 1) => JumpAndLink {
                destination: 1,
                offset: jump_immediate(value),
            },
            (1, 2) => Immediate {
                operation: ImmediateOperation::Add,
                destination: unsigned_register(value, 7),
                source: 0,
                immediate: compressed_immediate(value),
            },
            (1, 3) => {
                let destination = unsigned_register(value, 7);

                if destination == 2 {
                    let immediate = (bits(value, 12, 1) << 9)
                        | (bits(value, 6, 1) << 4)
                        | (bits(value, 5, 1) << 6)
                        | (bits(value, 3, 3) << 7)
                        | (bits(value, 2, 1) << 5);

                    if immediate == 0 {
                        return None;
                    }

                    Immediate {
                        operation: ImmediateOperation::Add,
                        destination: 2,
                        source: 2,
                        immediate: sign_extend(immediate, 10),
                    }
                } else {
                    if compressed_immediate(value) == 0 {
                        return None;
                    }

                    LoadUpperImmediate {
                        destination,
                        value: (compressed_immediate(value) as u32) << 12,
                    }
                }
            }
            (1, 4) => {
                let destination = prime_register(value, 7);
                let shift = bits(value, 10, 3);

                match shift {
                    0 | 1 => {
                        if bits(value, 12, 1) != 0 {
                            return None;
                        }

                        Immediate {
                            operation: if shift == 0 {
                                ImmediateOperation::ShiftRightLogical
                            } else {
                                ImmediateOperation::ShiftRightArithmetic
                            },
                            destination,
                            source: destination,
                            immediate: bits(value, 2, 31) as i32,
                        }
                    }
                    2 => Immediate {
                        operation: ImmediateOperation::And,
                        destination,
                        source: destination,
                        immediate: compressed_immediate(value),
                    },
                    _ => {
                        if bits(value, 12, 1) != 0 {
                            return None;
                        }

                        let operation = match bits(value, 5, 3) {
                            0 => RegisterOperation::Sub,
                            1 => RegisterOperation::Xor,
                            2 => RegisterOperation::Or,
                            _ => RegisterOperation::And,
                        };
                        Register {
                            operation,
                            destination,
                            first: destination,
                            second: prime_register(value, 2),
                        }
                    }
                }
            }
            (1, 5) => JumpAndLink {
                destination: 0,
                offset: jump_immediate(value),
            },
            (1, 6 | 7) => Branch {
                condition: if function_code == 6 {
                    BranchCondition::Equal
                } else {
                    BranchCondition::NotEqual
                },
                first: prime_register(value, 7),
                second: 0,
                offset: branch_immediate(value),
            },
            (2, 0) => {
                if bits(value, 12, 1) != 0 {
                    return None;
                }

                let destination = unsigned_register(value, 7);
                Immediate {
                    operation: ImmediateOperation::ShiftLeft,
                    destination,
                    source: destination,
                    immediate: bits(value, 2, 31) as i32,
                }
            }
            (2, 2) => {
                let destination = unsigned_register(value, 7);

                if destination == 0 {
                    return None;
                }

                let offset = (bits(value, 12, 1) << 5)
                    | (bits(value, 4, 7) << 2)
                    | (bits(value, 2, 3) << 6);
                Load {
                    kind: LoadKind::Word,
                    destination,
                    base: 2,
                    offset: offset as i32,
                }
            }
            (2, 4) => {
                let destination = unsigned_register(value, 7);
                let second = unsigned_register(value, 2);
                let higher_bit = bits(value, 12, 1) != 0;

                match (higher_bit, second == 0, destination == 0) {
                    (false, true, false) => JumpAndLinkRegister {
                        destination: 0,
                        base: destination,
                        offset: 0,
                    },
                    (false, true, true) => return None,
                    (false, false, _) => Register {
                        operation: RegisterOperation::Add,
                        destination,
                        first: 0,
                        second,
                    },
                    (true, true, true) => Breakpoint,
                    (true, true, false) => JumpAndLinkRegister {
                        destination: 1,
                        base: destination,
                        offset: 0,
                    },
                    (true, false, _) => Register {
                        operation: RegisterOperation::Add,
                        destination,
                        first: destination,
                        second,
                    },
                }
            }
            (2, 6) => {
                let offset =
                    (bits(value, 9, 15) << 2) | (bits(value, 7, 3) << 6);
                Store {
                    kind: StoreKind::Word,
                    source: unsigned_register(value, 2),
                    base: 2,
                    offset: offset as i32,
                }
            }
            _ => return None,
        }
    };
    Some(instruction)
}

fn compressed_immediate(value: u16) -> i32 {
    sign_extend((bits(value, 12, 1) << 5) | bits(value, 2, 31), 6)
}

fn doubleword(value: u16) -> Option<Option<Instruction>> {
    use Instruction::*;
    let higher_bit = bits(value, 12, 1) != 0;
    let instruction = match (value & 3, bits(value, 13, 7)) {
        (0, 3 | 7) => {
            let offset = (bits(value, 10, 7) << 3) | (bits(value, 5, 3) << 6);
            if bits(value, 13, 7) == 3 {
                Load {
                    kind: LoadKind::Doubleword,
                    destination: prime_register(value, 2),
                    base: prime_register(value, 7),
                    offset: offset as i32,
                }
            } else {
                Store {
                    kind: StoreKind::Doubleword,
                    source: prime_register(value, 2),
                    base: prime_register(value, 7),
                    offset: offset as i32,
                }
            }
        }
        (1, 1) => {
            let destination = unsigned_register(value, 7);
            if destination == 0 {
                return Some(None);
            }
            ImmediateWord {
                operation: ImmediateOperation::Add,
                destination,
                source: destination,
                immediate: compressed_immediate(value),
            }
        }
        (1, 4) if higher_bit && bits(value, 10, 3) < 2 => {
            let destination = prime_register(value, 7);
            Immediate {
                operation: if bits(value, 10, 3) == 0 {
                    ImmediateOperation::ShiftRightLogical
                } else {
                    ImmediateOperation::ShiftRightArithmetic
                },
                destination,
                source: destination,
                immediate: 32 + bits(value, 2, 31) as i32,
            }
        }
        (1, 4) if higher_bit && bits(value, 10, 3) == 3 => {
            let destination = prime_register(value, 7);
            RegisterWord {
                operation: match bits(value, 5, 3) {
                    0 => RegisterOperation::Sub,
                    1 => RegisterOperation::Add,
                    _ => return Some(None),
                },
                destination,
                first: destination,
                second: prime_register(value, 2),
            }
        }
        (2, 0) if higher_bit => {
            let destination = unsigned_register(value, 7);
            if destination == 0 {
                Hint
            } else {
                Immediate {
                    operation: ImmediateOperation::ShiftLeft,
                    destination,
                    source: destination,
                    immediate: 32 + bits(value, 2, 31) as i32,
                }
            }
        }
        (2, 3) => {
            let destination = unsigned_register(value, 7);
            if destination == 0 {
                return Some(None);
            }
            let offset = (bits(value, 12, 1) << 5)
                | (bits(value, 5, 3) << 3)
                | (bits(value, 2, 7) << 6);
            Load {
                kind: LoadKind::Doubleword,
                destination,
                base: 2,
                offset: offset as i32,
            }
        }
        (2, 7) => {
            let offset = (bits(value, 10, 7) << 3) | (bits(value, 7, 7) << 6);
            Store {
                kind: StoreKind::Doubleword,
                source: unsigned_register(value, 2),
                base: 2,
                offset: offset as i32,
            }
        }
        _ => return None,
    };
    Some(Some(instruction))
}

fn is_hint(value: u16) -> bool {
    let destination = unsigned_register(value, 7);
    let immediate = compressed_immediate(value);
    let shift = bits(value, 2, 31);

    match (value & 3, value >> 13) {
        (1, 0) => destination == 0 || immediate == 0,
        (1, 2) => destination == 0,
        (1, 3) => destination == 0 && immediate != 0,
        (1, 4) => {
            bits(value, 12, 1) == 0 && bits(value, 10, 3) < 2 && shift == 0
        }
        (2, 0) => bits(value, 12, 1) == 0 && (destination == 0 || shift == 0),
        (2, 4) => destination == 0 && shift != 0,
        _ => false,
    }
}

fn jump_immediate(value: u16) -> i32 {
    let result = (bits(value, 12, 1) << 11)
        | (bits(value, 11, 1) << 4)
        | (bits(value, 9, 3) << 8)
        | (bits(value, 8, 1) << 10)
        | (bits(value, 7, 1) << 6)
        | (bits(value, 6, 1) << 7)
        | (bits(value, 3, 7) << 1)
        | (bits(value, 2, 1) << 5);
    sign_extend(result, 12)
}

fn prime_register(value: u16, shift: u32) -> u8 {
    (bits(value, shift, 7) + 8) as u8
}

fn unsigned_register(value: u16, shift: u32) -> u8 {
    bits(value, shift, 31) as u8
}
