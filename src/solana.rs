use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::{Keypair, Signer};

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
}
