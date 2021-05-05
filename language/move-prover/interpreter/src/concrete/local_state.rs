// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_binary_format::{errors::PartialVMError, file_format::CodeOffset};
use move_core_types::vm_status::StatusCode;
use move_model::ast::TempIndex;

use crate::concrete::{
    ty::Type,
    value::{BaseValue, LocalSlot, Pointer, RefTypedValue, TypedValue},
};

#[derive(Clone, Debug)]
pub enum AbortInfo {
    User(u64),
    Internal(StatusCode),
}

impl AbortInfo {
    pub fn usr_abort(status_code: u64) -> Self {
        AbortInfo::User(status_code)
    }
    pub fn sys_abort(status_code: StatusCode) -> Self {
        AbortInfo::Internal(status_code)
    }

    pub fn into_err(self) -> PartialVMError {
        match self {
            Self::User(status_code) => {
                PartialVMError::new(StatusCode::ABORTED).with_sub_status(status_code)
            }
            Self::Internal(status_code) => PartialVMError::new(status_code),
        }
    }

    pub fn get_status_code(&self) -> u64 {
        match self {
            Self::User(status_code) => *status_code,
            Self::Internal(status_code) => *status_code as u64,
        }
    }
}

#[derive(Debug)]
pub enum TerminationStatus {
    None,
    PostAbort(AbortInfo),
    Return(Vec<TypedValue>),
    Abort(AbortInfo),
}

pub struct LocalState {
    /// slots that holds local variables
    slots: Vec<LocalSlot>,
    /// program counter
    pc: CodeOffset,
    /// whether we set the PC to branch in the handling of last bytecode
    pc_branch: bool,
    /// termination status
    termination: TerminationStatus,
}

impl LocalState {
    pub fn new(slots: Vec<LocalSlot>) -> Self {
        Self {
            slots,
            pc: 0,
            pc_branch: false,
            termination: TerminationStatus::None,
        }
    }

    pub fn get_type(&self, index: TempIndex) -> &Type {
        self.slots.get(index).unwrap().get_type()
    }

    pub fn get_value(&self, index: TempIndex) -> RefTypedValue {
        self.slots.get(index).unwrap().get_value()
    }
    pub fn put_value(&mut self, index: TempIndex, val: BaseValue, ptr: Pointer) {
        self.slots.get_mut(index).unwrap().put_value(val, ptr)
    }
    pub fn del_value(&mut self, index: TempIndex) -> TypedValue {
        self.slots.get_mut(index).unwrap().del_value()
    }

    pub fn get_pc(&self) -> CodeOffset {
        self.pc
    }
    pub fn set_pc(&mut self, pc: CodeOffset) {
        if cfg!(debug_assertions) {
            assert!(!self.pc_branch);
        }
        self.pc = pc;
        self.pc_branch = true;
    }
    pub fn ready_pc_for_next_instruction(&mut self) {
        if self.pc_branch {
            self.pc_branch = false
        } else {
            self.pc += 1;
        }
    }

    pub fn transit_to_post_abort(&mut self, info: AbortInfo) {
        if cfg!(debug_assertions) {
            assert!(matches!(self.termination, TerminationStatus::None));
        }
        self.termination = TerminationStatus::PostAbort(info);
    }
    pub fn is_terminated(&self) -> bool {
        matches!(
            self.termination,
            TerminationStatus::Return(_) | TerminationStatus::Abort(_)
        )
    }
    pub fn terminate_with_abort(&mut self, abort_code: u64) {
        if cfg!(debug_assertions) {
            assert!(!self.is_terminated());
        }
        let info = match &self.termination {
            TerminationStatus::None => AbortInfo::usr_abort(abort_code),
            TerminationStatus::PostAbort(original_info) => {
                if cfg!(debug_assertions) {
                    assert_eq!(original_info.get_status_code(), abort_code);
                }
                original_info.clone()
            }
            _ => unreachable!(),
        };
        self.termination = TerminationStatus::Abort(info);
    }
    pub fn terminate_with_return(&mut self, ret_vals: Vec<TypedValue>) {
        if cfg!(debug_assertions) {
            assert!(!self.is_terminated());
        }
        self.termination = TerminationStatus::Return(ret_vals);
    }
    pub fn into_termination_status(self) -> TerminationStatus {
        self.termination
    }
}