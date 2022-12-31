use sha2::Sha256;
use sha2::digest::{Digest, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use std::cmp::PartialEq;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ClientMessage {
    QueryLatest,
    QueryAll,
    ResponseLatest(Block),
    ResponseAll(Vec<Block>),
}

const GENESIS_DATA: &str = "genesis data";
const GENESIS_TIMESTAMP: u128 = 0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub prev_hash: [u8; 32],
    pub index: u32,
    pub timestamp: u128,
    pub data: String,
    pub hash: [u8; 32],
}

impl Block {
    fn calculate_hash(block: &Block) -> [u8; 32] {
        Sha256::new()
            .chain_update(&block.prev_hash)
            .chain_update(block.index.to_be_bytes())
            .chain_update(block.timestamp.to_be_bytes())
            .chain_update(&block.data)
            .finalize()
            .as_slice()
            .try_into()
            .expect("Wrong length")
    }

    fn new(
        prev_hash: [u8; 32], 
        index: u32,
        timestamp: u128,
        data: String
    ) -> Self {
        let mut block = Block { 
            prev_hash, 
            index, 
            timestamp, 
            data,
            hash: Output::<Sha256>::default().as_slice().try_into().expect("wrong length"),
        };
        block.hash = Self::calculate_hash(&block);
        block
    }

    fn generate_block(data: String, prev_block: &Block) -> Self {
        Self::new(
            prev_block.hash.clone(),
            prev_block.index + 1,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("went back in time")
                .as_millis(),
            data
        )
    }

    fn validate_block(&self, prev_block: &Block) -> bool {
        (self.index == prev_block.index + 1) &&
        (self.prev_hash == prev_block.hash) &&
        (self.hash == Self::calculate_hash(&self))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Handler {
    pub blockchain: Vec<Block>,
    genesis: Block,
}

impl Handler {
    pub fn new() -> Self {
        let genesis = Block::new( 
            Output::<Sha256>::default().as_slice().try_into().expect("Wrong length"),
            0,
            GENESIS_TIMESTAMP,
            String::from(GENESIS_DATA),
        );
        Handler {
            blockchain: vec![genesis.clone()],
            genesis, 
        }
    }

    fn validate_chain(genesis: &Block, chain: &Vec<Block>) -> bool {
        if chain.get(0).expect("chain empty") != genesis {
            return false;
        }
        let mut prev = None;
        for block in chain {
            if let Some(prev_block) = prev {
                if !block.validate_block(prev_block) { 
                    return false;
                }
            }
            prev = Some(&block);
        }
        true
    }

    pub fn replace_chain(&mut self, new_chain: Vec<Block>) -> bool{
        if Self::validate_chain(&self.genesis, &new_chain) && new_chain.len() > self.blockchain.len() {
            self.blockchain = new_chain;
            true
        } else {
            false
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.blockchain.last().expect("chain empty")
    }

    pub fn push_block(&mut self, new_block: Block) -> bool {
        if new_block.validate_block(self.latest_block()) {
            self.blockchain.push(new_block);
            true
        } else {
            false
        }
    }

    pub fn handle(&mut self, ser_in: &str) -> Option<String> {
        match serde_json::from_str(ser_in) {
            Ok(de_in) => {
                self.process(de_in)
                    .map(|de_out| {
                        serde_json::to_string(&de_out).unwrap()
                    })
            }
            Err(de_err) => {
                println!("de err: {}", de_err);
                None
            }
        }
    }

    pub fn process(&mut self, msg: ClientMessage) -> Option<ClientMessage> {
        match msg {
            ClientMessage::QueryLatest => 
                Some(
                    ClientMessage::ResponseLatest(
                        self.latest_block().clone()
                    )
                ),
            ClientMessage::QueryAll => 
                Some(
                    ClientMessage::ResponseAll(
                        self.blockchain.clone()
                    )
                ),
            ClientMessage::ResponseLatest(new_block) => {
                self.push_block(new_block);
                None
            },
            ClientMessage::ResponseAll(new_chain) => {
                self.replace_chain(new_chain);
                None
            }
        }
    }

}
