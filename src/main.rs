use sha2::{Digest, Sha256};
use chrono::Utc;
use std::collections::HashMap;
use rand::Rng;

const DIFFICULTY: usize = 4;
const MINING_REWARD: f64 = 100.0;
const HALVING_INTERVAL: u32 = 10;

#[derive(Clone, Debug)]
struct Transaction {
    from: String,
    to: String,
    amount: f64,
}

impl Transaction {
    fn new(from: String, to: String, amount: f64) -> Self {
        Self { from, to, amount }
    }
}

#[derive(Clone)]
struct Block {
    index: u32,
    timestamp: i64,
    transactions: Vec<Transaction>,
    previous_hash: String,
    hash: String,
    nonce: u32,
}

impl Block {
    fn new(index: u32, transactions: Vec<Transaction>, previous_hash: String) -> Block {
        let mut block = Block {
            index,
            timestamp: Utc::now().timestamp(),
            transactions,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.mine();
        block
    }

    fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let data = format!("{}{}{:?}{}{}", self.index, self.timestamp, &self.transactions, &self.previous_hash, self.nonce);
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn mine(&mut self) {
        let target = "0".repeat(DIFFICULTY);
        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }
        println!("Block mined: {}", self.hash);
    }
}

struct Blockchain {
    chain: Vec<Block>,
    pending_transactions: Vec<Transaction>,
    wallets: HashMap<String, f64>,
    current_mining_reward: f64,
}

impl Blockchain {
    fn new() -> Blockchain {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            wallets: HashMap::new(),
            current_mining_reward: MINING_REWARD,
        };
        blockchain.create_genesis_block();
        blockchain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block::new(0, vec![], String::from("0"));
        self.chain.push(genesis_block);
    }

    fn create_wallet(&mut self) -> String {
        let address = format!("0x{:x}", rand::thread_rng().gen::<u64>());
        self.wallets.insert(address.clone(), 0.0);
        address
    }

    fn get_balance(&self, address: &str) -> f64 {
        *self.wallets.get(address).unwrap_or(&0.0)
    }

    fn add_transaction(&mut self, transaction: Transaction) -> bool {
        if transaction.from != "0" && self.get_balance(&transaction.from) < transaction.amount {
            return false;
        }
        self.pending_transactions.push(transaction);
        true
    }

    fn mine_pending_transactions(&mut self, miner_address: &str) {
        let mut transactions_to_mine = self.pending_transactions.clone();

        for tx in &transactions_to_mine {
            if tx.from != "0" {
                *self.wallets.entry(tx.from.clone()).or_insert(0.0) -= tx.amount;
            }
            *self.wallets.entry(tx.to.clone()).or_insert(0.0) += tx.amount;
        }

        let reward_tx = Transaction::new(String::from("0"), miner_address.to_string(), self.current_mining_reward);
        transactions_to_mine.push(reward_tx);

        let new_block = Block::new(
            self.chain.len() as u32,
            transactions_to_mine,
            self.chain.last().unwrap().hash.clone(),
        );
        self.chain.push(new_block);

        *self.wallets.entry(miner_address.to_string()).or_insert(0.0) += self.current_mining_reward;

        self.pending_transactions.clear();

        if self.chain.len() as u32 % HALVING_INTERVAL == 0 {
            self.current_mining_reward /= 2.0;
            println!("Mining reward halved to {} tokens", self.current_mining_reward);
        }
    }

    fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }

            if !current_block.hash.starts_with(&"0".repeat(DIFFICULTY)) {
                return false;
            }
        }
        true
    }

    fn print_chain(&self) {
        for (i, block) in self.chain.iter().enumerate() {
            println!("Block #{}", i);
            println!("Hash: {}", block.hash);
            println!("Previous Hash: {}", block.previous_hash);
            println!("Transactions: {}", block.transactions.len());
            for (j, tx) in block.transactions.iter().enumerate() {
                println!("  Transaction {}: {} tokens from {} to {}", j+1, tx.amount, tx.from, tx.to);
            }
            println!();
        }
        println!("Blockchain validity: {}", self.is_chain_valid());
        println!("Current mining reward: {} tokens", self.current_mining_reward);
    }
}

fn main() {
    let mut blockchain = Blockchain::new();
    let mut wallets: Vec<String> = Vec::new();

    loop {
        println!("1. Create a new wallet");
        println!("2. Check wallet balance");
        println!("3. Send tokens");
        println!("4. Mine pending transactions");
        println!("5. View blockchain");
        println!("6. Exit");

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).expect("Failed to read line");

        match choice.trim() {
            "1" => {
                let new_wallet = blockchain.create_wallet();
                wallets.push(new_wallet.clone());
                println!("New wallet created: {}", new_wallet);
            }
            "2" => {
                if wallets.is_empty() {
                    println!("No wallets created yet. Create a wallet first.");
                } else {
                    for (i, wallet) in wallets.iter().enumerate() {
                        println!("{}. {}: {} tokens", i + 1, wallet, blockchain.get_balance(wallet));
                    }
                    print!("Enter the number of the wallet: ");
                    let mut wallet_choice = String::new();
                    std::io::stdin().read_line(&mut wallet_choice).expect("Failed to read line");
                    if let Ok(index) = wallet_choice.trim().parse::<usize>() {
                        if index > 0 && index <= wallets.len() {
                            let wallet = &wallets[index - 1];
                            println!("Balance of {}: {} tokens", wallet, blockchain.get_balance(wallet));
                        } else if index != 0 {
                            println!("Invalid wallet selection");
                        }
                    } else {
                        println!("Invalid input");
                    }
                }
            }
            "3" => {
                if wallets.len() < 2 {
                    println!("Need at least two wallets to make a transaction. Please create more wallets.");
                } else {
                    println!("Select sender wallet:");
                    for (i, wallet) in wallets.iter().enumerate() {
                        println!("{}. {}: {} tokens", i + 1, wallet, blockchain.get_balance(wallet));
                    }
                    print!("Choose sender (enter the number): ");
                    let mut sender_choice = String::new();
                    std::io::stdin().read_line(&mut sender_choice).expect("Failed to read line");
                    if let Ok(sender_index) = sender_choice.trim().parse::<usize>() {
                        if sender_index > 0 && sender_index <= wallets.len() {
                            let sender = wallets[sender_index - 1].clone();
                            
                            println!("Select recipient wallet:");
                            for (i, wallet) in wallets.iter().enumerate() {
                                if i != sender_index - 1 {
                                    println!("{}. {}", i + 1, wallet);
                                }
                            }
                            print!("Choose recipient (enter the number): ");
                            let mut recipient_choice = String::new();
                            std::io::stdin().read_line(&mut recipient_choice).expect("Failed to read line");
                            if let Ok(recipient_index) = recipient_choice.trim().parse::<usize>() {
                                if recipient_index > 0 && recipient_index <= wallets.len() && recipient_index != sender_index {
                                    let recipient = wallets[recipient_index - 1].clone();
                                    
                                    print!("Enter amount to send: ");
                                    let mut amount_str = String::new();
                                    std::io::stdin().read_line(&mut amount_str).expect("Failed to read line");
                                    if let Ok(amount) = amount_str.trim().parse::<f64>() {
                                        let transaction = Transaction::new(sender.clone(), recipient, amount);
                                        if blockchain.add_transaction(transaction) {
                                            println!("Transaction added to pending transactions");
                                            println!("Note: this txn will be processed when the next block is mined.");
                                        } else {
                                            println!("Transaction failed: Insufficient balance");
                                        }
                                    } else {
                                        println!("Invalid amount");
                                    }
                                } else {
                                    println!("Invalid recipient selection");
                                }
                            } else {
                                println!("Invalid input");
                            }
                        } else {
                            println!("Invalid sender selection");
                        }
                    } else {
                        println!("Invalid input");
                    }
                }
            }
            "4" => {
                if wallets.is_empty() {
                    println!("No wallets available for mining. Create a wallet first.");
                } else {
                    println!("Select a wallet for mining:");
                    for (i, wallet) in wallets.iter().enumerate() {
                        println!("{}. {}", i + 1, wallet);
                    }
                    print!("Choose miner (enter the number): ");
                    let mut miner_choice = String::new();
                    std::io::stdin().read_line(&mut miner_choice).expect("Failed to read line");
                    if let Ok(index) = miner_choice.trim().parse::<usize>() {
                        if index > 0 && index <= wallets.len() {
                            let miner = &wallets[index - 1];
                            blockchain.mine_pending_transactions(miner);
                            println!("Block mined and added to the blockchain");
                            println!("Miner {} received {} tokens as reward", miner, blockchain.current_mining_reward);
                        } else {
                            println!("Invalid miner selection");
                        }
                    } else {
                        println!("Invalid input");
                    }
                }
            }
            "5" => {
                blockchain.print_chain();
            }
            "6" => {
                println!("Exiting the Blockchain Simulator...");
                break;
            }
            _ => println!("Invalid option. Please choose a number between 1 and 6."),
        }
    }
}