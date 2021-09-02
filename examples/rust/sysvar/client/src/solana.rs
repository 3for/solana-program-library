
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_program::sysvar::{self};

use anyhow::{anyhow, bail, Context, Result};
use log::info;

pub struct Config {
    pub json_rpc_url: String,
    pub keypair: Keypair,
}

pub fn load_config() -> Result<Config> {
    let config_file = solana_cli_config::CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow!("config file path"))?;
    let cli_config = solana_cli_config::Config::load(&config_file)?;
    
    let json_rpc_url = cli_config.json_rpc_url;
    let keypair = read_keypair_file(&cli_config.keypair_path).map_err(|e| anyhow!("{}", e))?;
    Ok(Config {
        json_rpc_url,
        keypair,
    })
}

pub fn connect(config: &Config) -> Result<RpcClient> {
    info!("connecting to solana node at {}", config.json_rpc_url);
    let client =
        RpcClient::new_with_commitment(config.json_rpc_url.clone(), CommitmentConfig::confirmed());

    let version = client.get_version()?;
    info!("RPC version: {:?}", version);

    let account = client
        .get_account(&config.keypair.pubkey())
        .context("unable to get payer account")?;

    info!("payer account: {:?}", account);

    Ok(client)
}

static DEPLOY_PATH: &str = "target/deploy";
static PROGRAM_KEYPAIR_PATH: &str = "spl_example_sysvar-keypair.json";
//static PROGRAM_KEYPAIR_PATH: &str = "helloworld-keypair.json";

pub fn get_program_keypair(client: &RpcClient) -> Result<Keypair> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let deploy_path = format!("{}/../{}", manifest_dir, DEPLOY_PATH);
    let program_keypair_path = format!("{}/{}", deploy_path, PROGRAM_KEYPAIR_PATH);

    info!("loading program keypair from {}", program_keypair_path);

    let program_keypair = read_keypair_file(&program_keypair_path)
        .map_err(|e| anyhow!("{}", e))
        .context("unable to load program keypair")?;

    let program_id = program_keypair.pubkey();

    info!("program id: {}", program_id);

    let account = client
        .get_account(&program_id)
        .context("unable to get program account")?;

    info!("program account: {:?}", account);

    if !account.executable {
        bail!("solana account not executable");
    }

    Ok(program_keypair)
}

pub fn commit(
    config: &Config,
    client: &RpcClient,
    program: &Keypair,
) -> Result<()> {
    let inst = create_instruction(&program.pubkey())?;
    let mut tx = Transaction::new_with_payer(&[inst], Some(&config.keypair.pubkey()));
    let blockhash = client.get_recent_blockhash()?.0;
    tx.try_sign(&[&config.keypair], blockhash)?;
    let sig = client.send_and_confirm_transaction_with_spinner(&tx)?;

    info!("plant sig: {}", &sig);
    Ok(())
}

fn create_instruction(
    program_id: &Pubkey,
) -> Result<Instruction> {
    let data: Vec<u8> = Vec::new();
    Ok(Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(sysvar::clock::id(), false),
            AccountMeta::new(sysvar::rent::id(), false),
        ],
        data,
    })
}

