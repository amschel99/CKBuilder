use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, Debug, PartialEq)]
enum HashType {
    Data,
    Type,
}

#[derive(Clone, Debug)]
struct Script {
    code_hash: String,
    hash_type: HashType,
    args: Vec<u8>,
}

#[derive(Clone, Debug)]
struct Cell {
    capacity: u64,
    lock: Script,
    type_: Option<Script>,
    data: Vec<u8>,
}

const MIN_CAPACITY: u64 = 61;

impl Cell {
    fn occupied(&self) -> u64 {
        let mut bytes: u64 = 8;
        bytes += 32 + self.lock.args.len() as u64 + 1;
        if let Some(t) = &self.type_ {
            bytes += 32 + t.args.len() as u64 + 1;
        }
        bytes += self.data.len() as u64;
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct OutPoint {
    tx_hash: String,
    index: u32,
}

#[derive(Clone, Default)]
struct Witness {
    lock: Vec<u8>,
}

struct Transaction {
    inputs: Vec<OutPoint>,
    outputs: Vec<Cell>,
    witnesses: Vec<Witness>,
    fee: u64,
}

struct Ledger {
    live: HashMap<OutPoint, Cell>,
}

fn run_lock(cell: &Cell, witness: &Witness) -> Result<(), String> {
    match cell.lock.code_hash.as_str() {
        "always_success" => Ok(()),

        "sum_equals_8" => {
            if witness.lock.len() < 16 {
                return Err("witness too short, need two numbers".into());
            }
            let a = u64::from_le_bytes(witness.lock[0..8].try_into().unwrap());
            let b = u64::from_le_bytes(witness.lock[8..16].try_into().unwrap());
            if a + b == 8 {
                Ok(())
            } else {
                Err(format!("{a} + {b} = {} != 8", a + b))
            }
        }

        other => Err(format!("unknown lock code `{other}`")),
    }
}

fn amount(cell: &Cell) -> u64 {
    if cell.data.len() < 8 {
        return 0;
    }
    u64::from_le_bytes(cell.data[0..8].try_into().unwrap())
}

fn has_type(cell: &Cell, code: &str, args: &[u8]) -> bool {
    matches!(&cell.type_, Some(t) if t.code_hash == code && t.args == args)
}

fn run_type_groups(inputs: &[Cell], outputs: &[Cell]) -> Result<(), String> {
    let mut keys: BTreeSet<(String, Vec<u8>)> = BTreeSet::new();
    for c in inputs.iter().chain(outputs.iter()) {
        if let Some(t) = &c.type_ {
            keys.insert((t.code_hash.clone(), t.args.clone()));
        }
    }

    for (code, args) in keys {
        match code.as_str() {
            "udt" => {
                let in_sum: u64 = inputs.iter().filter(|c| has_type(c, &code, &args)).map(amount).sum();
                let out_sum: u64 = outputs.iter().filter(|c| has_type(c, &code, &args)).map(amount).sum();
                let token = String::from_utf8_lossy(&args);
                if in_sum != out_sum {
                    return Err(format!(
                        "UDT `{token}`: supply not conserved (in {in_sum} != out {out_sum})"
                    ));
                }
                println!("    type `udt` [{token}] ok: {in_sum} in == {out_sum} out");
            }
            other => return Err(format!("unknown type code `{other}`")),
        }
    }
    Ok(())
}

fn validate_and_apply(ledger: &mut Ledger, tx: &Transaction, new_tx_hash: &str) -> Result<(), String> {
    let mut input_cells = Vec::new();
    for op in &tx.inputs {
        match ledger.live.get(op) {
            Some(c) => input_cells.push(c.clone()),
            None => return Err(format!("input {op:?} is not a live cell (already spent?)")),
        }
    }

    let sum_in: u64 = input_cells.iter().map(|c| c.capacity).sum();
    let sum_out: u64 = tx.outputs.iter().map(|c| c.capacity).sum();
    if sum_in < sum_out + tx.fee {
        return Err(format!(
            "capacity doesn't balance: inputs {sum_in} < outputs {sum_out} + fee {}",
            tx.fee
        ));
    }
    println!("    capacity ok: {sum_in} in  >=  {sum_out} out + {} fee", tx.fee);

    for (i, c) in tx.outputs.iter().enumerate() {
        if c.capacity < MIN_CAPACITY {
            return Err(format!("output {i}: capacity {} below the {MIN_CAPACITY} minimum", c.capacity));
        }
        if c.occupied() > c.capacity {
            return Err(format!(
                "output {i}: needs {} bytes but capacity is only {}",
                c.occupied(),
                c.capacity
            ));
        }
    }

    for (i, c) in input_cells.iter().enumerate() {
        let w = tx.witnesses.get(i).cloned().unwrap_or_default();
        run_lock(c, &w).map_err(|e| format!("lock failed on input {i}: {e}"))?;
        println!("    lock ok on input {i} (`{}`)", c.lock.code_hash);
    }

    run_type_groups(&input_cells, &tx.outputs)?;

    for op in &tx.inputs {
        ledger.live.remove(op);
    }
    for (i, c) in tx.outputs.iter().enumerate() {
        ledger.live.insert(
            OutPoint { tx_hash: new_tx_hash.to_string(), index: i as u32 },
            c.clone(),
        );
    }
    Ok(())
}

fn lock(code: &str) -> Script {
    Script { code_hash: code.into(), hash_type: HashType::Type, args: vec![] }
}

fn udt_type(token: &str) -> Script {
    Script { code_hash: "udt".into(), hash_type: HashType::Type, args: token.as_bytes().to_vec() }
}

fn two_numbers(a: u64, b: u64) -> Witness {
    let mut v = a.to_le_bytes().to_vec();
    v.extend_from_slice(&b.to_le_bytes());
    Witness { lock: v }
}

fn run_tx(ledger: &mut Ledger, title: &str, tx: Transaction, new_hash: &str) {
    println!("\n=== {title} ===");
    match validate_and_apply(ledger, &tx, new_hash) {
        Ok(()) => println!("  ACCEPTED -> {} live cells now", ledger.live.len()),
        Err(e) => println!("  REJECTED: {e}"),
    }
}

fn main() {
    let mut ledger = Ledger { live: HashMap::new() };

    ledger.live.insert(
        OutPoint { tx_hash: "genesis".into(), index: 0 },
        Cell { capacity: 1000, lock: lock("sum_equals_8"), type_: None, data: vec![] },
    );
    ledger.live.insert(
        OutPoint { tx_hash: "genesis".into(), index: 1 },
        Cell {
            capacity: 300,
            lock: lock("always_success"),
            type_: Some(udt_type("TOKEN_A")),
            data: 100u64.to_le_bytes().to_vec(),
        },
    );
    println!("Genesis: {} live cells", ledger.live.len());

    run_tx(
        &mut ledger,
        "TX1: spend puzzle cell with 2 + 5 (should fail the lock)",
        Transaction {
            inputs: vec![OutPoint { tx_hash: "genesis".into(), index: 0 }],
            outputs: vec![Cell { capacity: 900, lock: lock("always_success"), type_: None, data: vec![] }],
            witnesses: vec![two_numbers(2, 5)],
            fee: 100,
        },
        "tx1",
    );

    run_tx(
        &mut ledger,
        "TX2: spend puzzle cell with 3 + 5 (lock passes)",
        Transaction {
            inputs: vec![OutPoint { tx_hash: "genesis".into(), index: 0 }],
            outputs: vec![Cell { capacity: 900, lock: lock("always_success"), type_: None, data: vec![] }],
            witnesses: vec![two_numbers(3, 5)],
            fee: 100,
        },
        "tx2",
    );

    run_tx(
        &mut ledger,
        "TX3: token transfer but output capacity = 40 (below the 61 floor)",
        Transaction {
            inputs: vec![OutPoint { tx_hash: "genesis".into(), index: 1 }],
            outputs: vec![Cell {
                capacity: 40,
                lock: lock("always_success"),
                type_: Some(udt_type("TOKEN_A")),
                data: 100u64.to_le_bytes().to_vec(),
            }],
            witnesses: vec![Witness::default()],
            fee: 0,
        },
        "tx3",
    );

    run_tx(
        &mut ledger,
        "TX4: token transfer minting 100 -> 120 (type script should block it)",
        Transaction {
            inputs: vec![OutPoint { tx_hash: "genesis".into(), index: 1 }],
            outputs: vec![Cell {
                capacity: 300,
                lock: lock("always_success"),
                type_: Some(udt_type("TOKEN_A")),
                data: 120u64.to_le_bytes().to_vec(),
            }],
            witnesses: vec![Witness::default()],
            fee: 0,
        },
        "tx4",
    );

    run_tx(
        &mut ledger,
        "TX5: split 100 token into 60 + 40 (conserved, should pass)",
        Transaction {
            inputs: vec![OutPoint { tx_hash: "genesis".into(), index: 1 }],
            outputs: vec![
                Cell {
                    capacity: 150,
                    lock: lock("always_success"),
                    type_: Some(udt_type("TOKEN_A")),
                    data: 60u64.to_le_bytes().to_vec(),
                },
                Cell {
                    capacity: 150,
                    lock: lock("always_success"),
                    type_: Some(udt_type("TOKEN_A")),
                    data: 40u64.to_le_bytes().to_vec(),
                },
            ],
            witnesses: vec![Witness::default()],
            fee: 0,
        },
        "tx5",
    );

    run_tx(
        &mut ledger,
        "TX6: spend the genesis token cell again (it's dead now)",
        Transaction {
            inputs: vec![OutPoint { tx_hash: "genesis".into(), index: 1 }],
            outputs: vec![Cell {
                capacity: 150,
                lock: lock("always_success"),
                type_: Some(udt_type("TOKEN_A")),
                data: 100u64.to_le_bytes().to_vec(),
            }],
            witnesses: vec![Witness::default()],
            fee: 0,
        },
        "tx6",
    );

    println!("\nFinal ledger: {} live cells", ledger.live.len());
}
