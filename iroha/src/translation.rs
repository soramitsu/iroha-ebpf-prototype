//! Functions taken from solana:
//! https://github.com/solana-labs/solana/blob/9be988db41c03cc644417584fb9bbee75554da14/programs/bpf_loader/src/syscalls.rs

use super::I2Error;
use solana_rbpf::{
    error::EbpfError,
    memory_region::{AccessType, MemoryMapping},
};

fn translate(
    memory_mapping: &MemoryMapping,
    access_type: AccessType,
    vm_addr: u64,
    len: u64,
) -> Result<u64, EbpfError<I2Error>> {
    memory_mapping.map::<I2Error>(access_type, vm_addr, len)
}

fn slice_inner<'a, T>(
    memory_mapping: &MemoryMapping,
    access_type: AccessType,
    vm_addr: u64,
    len: u64,
    enforce_aligned_host_addrs: bool,
) -> Result<&'a mut [T], EbpfError<I2Error>> {
    if !enforce_aligned_host_addrs
        && (vm_addr as u64 as *mut T).align_offset(std::mem::align_of::<T>()) != 0
    {
        Err(I2Error(format!("UnalignedPointer")))?;
    }
    if len == 0 {
        return Ok(&mut []);
    }

    let host_addr = translate(
        memory_mapping,
        access_type,
        vm_addr,
        len.saturating_mul(std::mem::size_of::<T>() as u64),
    )?;

    if enforce_aligned_host_addrs
        && (host_addr as *mut T).align_offset(std::mem::align_of::<T>()) != 0
    {
        Err(I2Error(format!("UnalignedPointer")))?;
    }
    Ok(unsafe { std::slice::from_raw_parts_mut(host_addr as *mut T, len as usize) })
}

pub fn slice<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    enforce_aligned_host_addrs: bool,
) -> Result<&'a [T], EbpfError<I2Error>> {
    slice_inner::<T>(
        memory_mapping,
        AccessType::Load,
        vm_addr,
        len,
        enforce_aligned_host_addrs,
    )
    .map(|value| &*value)
}

pub fn slice_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    len: u64,
    enforce_aligned_host_addrs: bool,
) -> Result<&'a mut [T], EbpfError<I2Error>> {
    slice_inner::<T>(
        memory_mapping,
        AccessType::Store,
        vm_addr,
        len,
        enforce_aligned_host_addrs,
    )
}

pub fn string_and_do(
    memory_mapping: &MemoryMapping,
    addr: u64,
    len: u64,
    enforce_aligned_host_addrs: bool,
    mut work: impl FnMut(&str) -> Result<u64, EbpfError<I2Error>>,
) -> Result<u64, EbpfError<I2Error>> {
    let buf = slice::<u8>(memory_mapping, addr, len, enforce_aligned_host_addrs)?;
    let i = match buf.iter().position(|byte| *byte == 0) {
        Some(i) => i,
        None => len as usize,
    };
    match std::str::from_utf8(&buf[..i]) {
        Ok(message) => work(message),
        Err(err) => Err(I2Error(format!("String is invalid: {}", err)).into()),
    }
}

fn type_inner<'a, T>(
    memory_mapping: &MemoryMapping,
    access_type: AccessType,
    vm_addr: u64,
    enforce_aligned_host_addrs: bool,
) -> Result<&'a mut T, EbpfError<I2Error>> {
    if !enforce_aligned_host_addrs
        && (vm_addr as *mut T).align_offset(std::mem::align_of::<T>()) != 0
    {
        Err(I2Error(format!("UnalignedPointer")))?;
    }

    let host_addr = translate(
        memory_mapping,
        access_type,
        vm_addr,
        std::mem::size_of::<T>() as u64,
    )?;

    if enforce_aligned_host_addrs
        && (host_addr as *mut T).align_offset(std::mem::align_of::<T>()) != 0
    {
        Err(I2Error(format!("UnalignedPointer")))?;
    }
    Ok(unsafe { &mut *(host_addr as *mut T) })
}

pub fn type_mut<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    enforce_aligned_host_addrs: bool,
) -> Result<&'a mut T, EbpfError<I2Error>> {
    type_inner::<T>(
        memory_mapping,
        AccessType::Store,
        vm_addr,
        enforce_aligned_host_addrs,
    )
}

pub fn r#type<'a, T>(
    memory_mapping: &MemoryMapping,
    vm_addr: u64,
    enforce_aligned_host_addrs: bool,
) -> Result<&'a T, EbpfError<I2Error>> {
    type_inner::<T>(
        memory_mapping,
        AccessType::Load,
        vm_addr,
        enforce_aligned_host_addrs,
    )
    .map(|value| &*value)
}

//pub fn program_address_inputs<'a>(
//    seeds_addr: u64,
//    seeds_len: u64,
//    program_id_addr: u64,
//    memory_mapping: &MemoryMapping,
//    loader_id: &Pubkey,
//    enforce_aligned_host_addrs: bool,
//) -> Result<(Vec<&'a [u8]>, &'a Pubkey), EbpfError<BpfError>> {
//    let untranslated_seeds = slice::<&[&u8]>(
//        memory_mapping,
//        seeds_addr,
//        seeds_len,
//        loader_id,
//        enforce_aligned_host_addrs,
//    )?;
//    if untranslated_seeds.len() > MAX_SEEDS {
//        return Err(SyscallError::BadSeeds(PubkeyError::MaxSeedLengthExceeded).into());
//    }
//    let seeds = untranslated_seeds
//        .iter()
//        .map(|untranslated_seed| {
//            slice::<u8>(
//                memory_mapping,
//                untranslated_seed.as_ptr() as *const _ as u64,
//                untranslated_seed.len() as u64,
//                loader_id,
//                enforce_aligned_host_addrs,
//            )
//        })
//        .collect::<Result<Vec<_>, EbpfError<BpfError>>>()?;
//    let program_id = type::<Pubkey>(
//        memory_mapping,
//        program_id_addr,
//        loader_id,
//        enforce_aligned_host_addrs,
//    )?;
//    Ok((seeds, program_id))
//}
