use std::time::Instant;

use color_eyre::Result;
use iroha::{client::Client, Iroha, Transaction};

fn main() -> Result<()> {
    let use_jit = false;
    let enforce_aligned_host_addrs = false;
    let max_gas = 10000;

    color_eyre::install()?;

    let iroha = Iroha::new(
        [("alice", 10), ("bob", 7)],
        use_jit,
        enforce_aligned_host_addrs,
        max_gas,
    );
    println!("WSV before EBPF execution: {:?}", iroha.wsv);

    let client = Client::new(&iroha);

    let file = std::env::args().nth(1).expect("Give file as argument");
    let tx = Transaction::from_file(file)?;

    const TIMES: u32 = 5000;

    let time = Instant::now();
    for _ in 0..TIMES {
        client.submit_transaction(&tx, "alice")?;
    }
    dbg!(time.elapsed() / TIMES);

    println!("WSV after EBPF execution: {:?}", iroha.wsv);
    Ok(())
}
