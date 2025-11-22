use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Keypair;

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

    pub async fn test_connection(&self) {
        let version = self.rpc_client.get_version().unwrap();
        println!("Connected to Solana devnet!");
        println!("Version: {:?}", version);
    }
}
