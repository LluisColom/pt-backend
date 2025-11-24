use crate::crypto::reading_hash;
use crate::db::SensorReading;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::UiTransactionEncoding;
use solana_client::rpc_response::OptionSerializer;
use solana_sdk::message::{AccountMeta, Instruction};
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use std::str::FromStr;

pub struct SolanaClient {
    pub rpc_client: RpcClient,
    pub keypair: Keypair,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, keypair: &str) -> anyhow::Result<Self> {
        // Read keypair from JSON
        let keypair_bytes: Vec<u8> = serde_json::from_str(keypair)?;
        let keypair = Keypair::try_from(keypair_bytes.as_slice())?;
        // Initialize RPC client
        let rpc_client = RpcClient::new(rpc_url.to_string());
        Ok(Self {
            rpc_client,
            keypair,
        })
    }

    /// Solana RPC connection sanity check
    pub async fn test_connection(&self) -> anyhow::Result<()> {
        let version = self.rpc_client.get_version()?;
        println!("Solana client version: {:?}", version);
        Ok(())
    }

    /// Checks the available balance of the linked wallet
    /// A minimum balance is required to issue transactions to Solana
    pub fn enough_balance(&self) -> anyhow::Result<bool> {
        let balance = self.rpc_client.get_balance(&self.keypair.pubkey())?;
        //println!("Balance: {:?} SOL", balance as f64 / 1_000_000_000.0);
        Ok(balance > 1_000_000) // 0.001 SOL minimum
    }

    pub async fn submit(&self, sensor_reading: SensorReading) -> anyhow::Result<String> {
        // Create memo with hash
        let hash = reading_hash(sensor_reading);
        let memo_data = format!("pollution:v1:{}", hash);

        // Memo program ID on mainnet/devnet
        let program_id = solana_sdk::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

        // Build memo instruction manually
        let memo_ix = Instruction {
            program_id,
            accounts: vec![AccountMeta::new_readonly(self.keypair.pubkey(), true)],
            data: memo_data.as_bytes().to_vec(),
        };

        // Get recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        let tx = Transaction::new_signed_with_payer(
            &[memo_ix],
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            recent_blockhash,
        );

        // Option 1: Fire and forget (faster, but less reliable)
        let signature = tx.signatures[0].to_string();
        self.rpc_client.send_transaction(&tx)?;

        // Option 2: Wait for confirmation (catches errors)
        //self.rpc_client.send_and_confirm_transaction(&tx)?;

        Ok(signature)
    }

    pub async fn verify(&self, reading: SensorReading, signature: String) -> anyhow::Result<bool> {
        // Calculate expected memo
        let hash = reading_hash(reading);
        let expected_memo = format!("pollution:v1:{}", hash);

        // Read transaction from blockchain
        let signature = Signature::from_str(&signature)?;
        let tx = self
            .rpc_client
            .get_transaction(&signature, UiTransactionEncoding::Json)?;

        // Extract memo from transaction
        if let Some(meta) = tx.transaction.meta {
            if let OptionSerializer::Some(log_messages) = meta.log_messages {
                for log in log_messages {
                    // Memo program logs look like: "Program log: Memo (len 32): \"pollution:v1:...\""
                    if log.contains(&expected_memo) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}
