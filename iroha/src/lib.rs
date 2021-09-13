use std::{cell::RefCell, collections::HashMap, error, fmt, path::Path, rc::Rc};

use color_eyre::Result;
use solana_rbpf::{
    error::UserDefinedError,
    vm::{Config, EbpfVm, Executable, ProgramResult, SyscallRegistry, TestInstructionMeter},
};

pub mod client;
mod syscalls;
pub mod translation;

#[derive(Clone, Debug)]
pub struct I2Error(pub String);

impl fmt::Display for I2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl error::Error for I2Error {}
impl UserDefinedError for I2Error {}

struct EBPFRuntime {
    executable: Box<dyn Executable<I2Error, TestInstructionMeter>>,
    use_jit: bool,
    max_gas: u64,
}

impl EBPFRuntime {
    fn get_syscalls() -> SyscallRegistry {
        let mut reg = SyscallRegistry::default();
        syscalls::register(&mut reg);
        reg
    }

    fn get_config() -> Config {
        Config {
            enable_instruction_meter: true,
            sanitize_user_provided_values: true,
            instruction_meter_checkpoint_distance: 100,
            ..Config::default()
        }
    }

    pub fn new(elf_file: impl AsRef<[u8]>, use_jit: bool, max_gas: u64) -> Result<Self> {
        let mut executable = <dyn Executable<I2Error, TestInstructionMeter>>::from_elf(
            elf_file.as_ref(),
            None, // verifier
            Self::get_config(),
            Self::get_syscalls(),
        )?;
        if use_jit {
            executable.jit_compile()?;
        }
        Ok(Self {
            executable,
            use_jit,
            max_gas,
        })
    }

    pub fn execute(&mut self, iroha: &Iroha, account_name: &str) -> ProgramResult<I2Error> {
        let mut params = IntoIterator::into_iter((account_name.len() as u32).to_le_bytes())
            .chain(account_name.as_bytes().iter().copied())
            .collect::<Vec<_>>();

        let mut vm = EbpfVm::<I2Error, TestInstructionMeter>::new(
            self.executable.as_ref(),
            &mut [], // heap
            &mut params,
        )
        .unwrap();
        syscalls::bind(
            &mut vm,
            iroha.enforce_aligned_host_addrs,
            Rc::clone(&iroha.wsv),
        )?;
        let mut meter = TestInstructionMeter {
            remaining: self.max_gas,
        };
        if self.use_jit {
            vm.execute_program_jit(&mut meter)
        } else {
            vm.execute_program_interpreted(&mut meter)
        }
    }
}

pub struct Iroha {
    pub wsv: Rc<RefCell<WSV>>,
    use_jit: bool,
    enforce_aligned_host_addrs: bool,
    max_gas: u64,
}

impl Iroha {
    pub fn new<'a>(
        accounts: impl IntoIterator<Item = (&'a str, u64)>,
        use_jit: bool,
        enforce_aligned_host_addrs: bool,
        max_gas: u64,
    ) -> Iroha {
        let wsv = WSV {
            accounts: accounts
                .into_iter()
                .map(|(name, balance)| (String::from(name).into_boxed_str(), Account { balance }))
                .collect(),
        };
        Self {
            wsv: Rc::new(RefCell::new(wsv)),
            use_jit,
            enforce_aligned_host_addrs,
            max_gas,
        }
    }
}

pub struct Transaction {
    /// Raw program (elf file)
    pub content: Vec<u8>,
}

impl Transaction {
    pub fn from_file(file: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read(file)?;
        Ok(Self::new(content))
    }

    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }

    pub fn execute(&self, iroha: &Iroha, account_name: &str) -> Result<()> {
        let mut runtime = EBPFRuntime::new(&self.content, iroha.use_jit, iroha.max_gas)?;
        runtime.execute(&iroha, account_name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Account {
    balance: u64,
}

#[derive(Debug, Clone, Default)]
pub struct WSV {
    pub accounts: HashMap<Box<str>, Account>,
}

impl WSV {
    pub fn mint(&mut self, account: &str, amount: u64) {
        if let Some(acc) = self.accounts.get_mut(account) {
            acc.balance += amount;
        }
    }

    pub fn burn(&mut self, account: &str, amount: u64) {
        if let Some(acc) = self.accounts.get_mut(account) {
            acc.balance -= amount;
        }
    }

    pub fn balance(&self, account: &str) -> Option<u64> {
        self.accounts.get(account).map(|a| a.balance)
    }
}
