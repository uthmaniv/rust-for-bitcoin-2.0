
use serde::{Serialize, Serializer};



#[derive(Debug, Serialize)]
pub struct Transaction {
    pub transaction_id: Txid,
    pub version: u32,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    pub lock_time: u32,
}


#[derive(Debug, Serialize)]
pub struct Input {
    pub txid: Txid, // [u8; 32],
    pub output_index: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32
}

#[derive(Debug, Serialize)]
pub struct Output {
    #[serde(serialize_with = "as_btc")]
    pub amount: Amount,
    pub script_pubkey: Vec<u8>,
}

fn as_btc<S: Serializer, T: BitcoinValue>(t: &T, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_f64(t.to_btc())
}

#[derive(Debug)]
pub struct Amount( u64);

impl Amount {
  // type associated functiion that initiate the instance of the struct i.e Amount
  pub fn from_sat(satoshi: u64) -> Amount {
    Amount(satoshi)
  }

  pub fn to_sat(&self) -> u64 {
    self.0
  }
}

#[derive(Debug)]
pub struct Txid([u8; 32]);

// [u8; 32] => array of 32 element each element is 1 byte [u8]; i.e one byte is u8;

impl Txid {
    pub fn from_bytes(bytes: [u8; 32]) -> Txid {
        Txid(bytes)
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        self.0
    }
}

impl Serialize for Txid {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        // Txids are conventionally displayed in big-endian (byte-reversed) hex.
        let mut bytes = self.0;
        bytes.reverse();
        s.serialize_str(&hex::encode(bytes))
    }
}


trait BitcoinValue {
    fn to_btc(&self) -> f64;
}

impl BitcoinValue for Amount {
    fn to_btc(&self) -> f64 {
        self.0 as f64 / 100_000_000.0
    }
}






