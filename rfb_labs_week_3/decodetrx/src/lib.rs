use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Error, Read};
use sha2::{Digest, Sha256};
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

/// Reads the 4-byte little-endian version directly from a raw transaction hex string.
pub fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(transaction_hex).expect("failed to decode hex");
    let mut slice: &[u8] = &bytes;
    read_u32(&mut slice).expect("failed to read version")
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    transaction_bytes
        .read_u64::<LittleEndian>()
        .expect("failed to read u64")
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    Ok(bytes_slice.read_u32::<LittleEndian>()?)
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let prefix = transaction_bytes.read_u8()?;
    match prefix {
        0x00..=0xfc => Ok(prefix as u64),
        0xfd => Ok(transaction_bytes.read_u16::<LittleEndian>()? as u64),
        0xfe => Ok(transaction_bytes.read_u32::<LittleEndian>()? as u64),
        _ => Ok(transaction_bytes.read_u64::<LittleEndian>()?),
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut bytes = [0u8; 32];
    transaction_bytes.read_exact(&mut bytes)?;
    Ok(Txid::from_bytes(bytes))
}

/// Reads a CompactSize-prefixed script and returns it as a hex string.
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let len = read_compact_size(transaction_bytes)? as usize;
    let mut script = vec![0u8; len];
    transaction_bytes.read_exact(&mut script)?;
    Ok(hex::encode(script))
}

/// Reads the 4-byte little-endian version from the start of the raw transaction bytes.
fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    Ok(transaction_bytes.read_u32::<LittleEndian>()?)
}

/// Computes the transaction id as the double SHA-256 of the serialized (non-witness) bytes.
fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first = Sha256::digest(row_transaction_bytes);
    let second = Sha256::digest(first);
    let mut id = [0u8; 32];
    id.copy_from_slice(&second);
    Ok(Txid::from_bytes(id))
}

/// Encodes `n` as a CompactSize integer and appends it to `buf`.
fn push_compact_size(buf: &mut Vec<u8>, n: u64) {
    match n {
        0..=0xfc => buf.push(n as u8),
        0xfd..=0xffff => {
            buf.push(0xfd);
            buf.extend_from_slice(&(n as u16).to_le_bytes());
        }
        0x10000..=0xffff_ffff => {
            buf.push(0xfe);
            buf.extend_from_slice(&(n as u32).to_le_bytes());
        }
        _ => {
            buf.push(0xff);
            buf.extend_from_slice(&n.to_le_bytes());
        }
    }
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw = hex::decode(transaction_hex)?;
    let mut bytes: &[u8] = &raw;

    let version = read_version_byte(&mut bytes)?;

    // The txid is computed over the serialized transaction without the SegWit
    // marker, flag, or witness data, so we rebuild that stripped serialization.
    let mut stripped: Vec<u8> = Vec::new();
    stripped.extend_from_slice(&version.to_le_bytes());

    // Detect SegWit: a 0x00 marker followed by a non-zero flag byte.
    let mut segwit = false;
    if bytes.first() == Some(&0x00) && bytes.get(1).map_or(false, |&f| f != 0x00) {
        segwit = true;
        let mut marker_flag = [0u8; 2];
        bytes.read_exact(&mut marker_flag)?;
    }

    let input_count = read_compact_size(&mut bytes)?;
    push_compact_size(&mut stripped, input_count);

    let mut inputs = Vec::new();
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes)?;
        stripped.extend_from_slice(&txid.as_bytes());

        let output_index = read_u32(&mut bytes)?;
        stripped.extend_from_slice(&output_index.to_le_bytes());

        let script_sig = hex::decode(read_script_size(&mut bytes)?)?;
        push_compact_size(&mut stripped, script_sig.len() as u64);
        stripped.extend_from_slice(&script_sig);

        let sequence = read_u32(&mut bytes)?;
        stripped.extend_from_slice(&sequence.to_le_bytes());

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut bytes)?;
    push_compact_size(&mut stripped, output_count);

    let mut outputs = Vec::new();
    for _ in 0..output_count {
        let amount = read_amount(&mut bytes)?;
        stripped.extend_from_slice(&amount.to_sat().to_le_bytes());

        let script_pubkey = hex::decode(read_script_size(&mut bytes)?)?;
        push_compact_size(&mut stripped, script_pubkey.len() as u64);
        stripped.extend_from_slice(&script_pubkey);

        outputs.push(Output { amount, script_pubkey });
    }

    // Each input has its own witness stack of CompactSize-prefixed items.
    if segwit {
        for _ in 0..input_count {
            let item_count = read_compact_size(&mut bytes)?;
            for _ in 0..item_count {
                let _item = hex::decode(read_script_size(&mut bytes)?)?;
            }
        }
    }

    let lock_time = read_u32(&mut bytes)?;
    stripped.extend_from_slice(&lock_time.to_le_bytes());

    let transaction_id = hash_row_transaction(&stripped)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
