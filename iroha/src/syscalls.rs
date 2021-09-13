use std::{cell::RefCell, rc::Rc};

use solana_rbpf::{
    error::EbpfError,
    memory_region::MemoryMapping,
    question_mark,
    vm::{EbpfVm, InstructionMeter, ProgramResult, SyscallObject, SyscallRegistry},
};

use crate::{translation, I2Error, WSV};

pub struct Mint {
    enforce_aligned_host_addrs: bool,
    wsv: Rc<RefCell<WSV>>,
}

impl Mint {
    fn run(&mut self, account: &str, amount: u64) {
        self.wsv.borrow_mut().mint(account, amount)
    }
}

impl SyscallObject<I2Error> for Mint {
    fn call(
        &mut self,
        account: u64,
        len: u64,
        amount: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &MemoryMapping,
        result: &mut ProgramResult<I2Error>,
    ) {
        *result = translation::string_and_do(
            memory_mapping,
            account,
            len,
            self.enforce_aligned_host_addrs,
            |account| {
                self.run(account, amount);
                Ok(0)
            },
        )
    }
}

pub struct Burn {
    enforce_aligned_host_addrs: bool,
    wsv: Rc<RefCell<WSV>>,
}

impl Burn {
    fn run(&mut self, account: &str, amount: u64) {
        self.wsv.borrow_mut().burn(account, amount)
    }
}

impl SyscallObject<I2Error> for Burn {
    fn call(
        &mut self,
        account: u64,
        len: u64,
        amount: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &MemoryMapping,
        result: &mut ProgramResult<I2Error>,
    ) {
        *result = translation::string_and_do(
            memory_mapping,
            account,
            len,
            self.enforce_aligned_host_addrs,
            |account| {
                self.run(account, amount);
                Ok(0)
            },
        )
    }
}

pub struct Balance {
    wsv: Rc<RefCell<WSV>>,
    enforce_aligned_host_addrs: bool,
}

impl Balance {
    fn run(&mut self, account: &str) -> u64 {
        self.wsv.borrow().balance(account).unwrap_or_default()
    }
}

impl SyscallObject<I2Error> for Balance {
    fn call(
        &mut self,
        account: u64,
        len: u64,
        amount_addr: u64,
        _arg4: u64,
        _arg5: u64,
        memory_mapping: &MemoryMapping,
        result: &mut ProgramResult<I2Error>,
    ) {
        let addr = question_mark!(
            translation::type_mut(memory_mapping, amount_addr, self.enforce_aligned_host_addrs),
            result
        );
        *result = translation::string_and_do(
            memory_mapping,
            account,
            len,
            self.enforce_aligned_host_addrs,
            |account| {
                *addr = self.run(account);
                Ok(0)
            },
        )
    }
}

pub fn register(registry: &mut SyscallRegistry) {
    registry
        .register_syscall_by_name(b"sol_log_", Mint::call)
        .unwrap();
    registry
        .register_syscall_by_name(b"sol_log_64_", Burn::call)
        .unwrap();
    registry
        .register_syscall_by_name(b"sol_log_compute_units_", Balance::call)
        .unwrap();
}

pub fn bind<I: InstructionMeter>(
    vm: &mut EbpfVm<I2Error, I>,
    enforce_aligned_host_addrs: bool,
    wsv: Rc<RefCell<WSV>>,
) -> Result<(), EbpfError<I2Error>> {
    vm.bind_syscall_context_object(
        Box::new(Mint {
            wsv: wsv.clone(),
            enforce_aligned_host_addrs,
        }),
        None,
    )?;
    vm.bind_syscall_context_object(
        Box::new(Burn {
            wsv: wsv.clone(),
            enforce_aligned_host_addrs,
        }),
        None,
    )?;
    vm.bind_syscall_context_object(
        Box::new(Balance {
            wsv,
            enforce_aligned_host_addrs,
        }),
        None,
    )?;
    Ok(())
}
