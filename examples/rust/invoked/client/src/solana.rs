
use std::thread::AccessError;

use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{read_keypair_file, Keypair, Signer};
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;

use anyhow::{anyhow, bail, Context, Result};
use log::info;
use std::convert::TryInto;
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

pub fn get_program_address(seeds: &[&[u8]], program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(seeds, &program_id)
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

pub fn get_specific_program_instance_account(
    client: &RpcClient,
    payer_account: &Keypair,
    program_keypair: &Keypair,
    lamports: u64,
    space: u64,
    pubkey: &Pubkey,
) -> Result<()> {
    info!("specific program account pubkey: {}", pubkey);

    let account = client.get_account(&pubkey);

    if account.is_err() {
        info!("creating specific program instance at {}", pubkey);

        let instr = system_instruction::create_account(
            &payer_account.pubkey(),
            &pubkey,
            lamports,
            space, //contract_size.try_into().expect("u64"),
            &program_keypair.pubkey(),
        );

        let recent_blockhash = client.get_recent_blockhash()?.0;
        info!("recent blockhash: {}", recent_blockhash);

        let tx = Transaction::new_signed_with_payer(
            &[instr],
            Some(&payer_account.pubkey()),
            &[payer_account],
            recent_blockhash,
        );

        let sig = client.send_and_confirm_transaction_with_spinner(&tx)?;

        info!("account created");
        info!("signature: {}", sig);
    }

    let account = client
        .get_account(&pubkey)
        .context("unable to get program instance account")?;

    info!("program instance account: {:?}", account);

    Ok(())
}


pub fn get_program_instance_account(
    client: &RpcClient,
    payer_account: &Keypair,
    program_keypair: &Keypair,
    seed: &str,
) -> Result<Pubkey> {
    //static SEED: &str = "transfertest";

    let pubkey =
        Pubkey::create_with_seed(&payer_account.pubkey(), seed, &program_keypair.pubkey())?;

    info!("program account pubkey: {}", pubkey);

    let account = client.get_account(&pubkey);

    if account.is_err() {
        info!("creating program instance at {}", pubkey);

        let contract_size = get_contract_size(client)?;
        info!("contract size: {}", contract_size);
        let lamports = client.get_minimum_balance_for_rent_exemption(contract_size)?;
        info!("minimim balance for rent exemption: {}", lamports);

        let instr = system_instruction::create_account_with_seed(
            &payer_account.pubkey(),
            &pubkey,
            &payer_account.pubkey(),
            seed,
            lamports,
            10,
            &program_keypair.pubkey(),
        );

        let recent_blockhash = client.get_recent_blockhash()?.0;
        info!("recent blockhash: {}", recent_blockhash);

        let tx = Transaction::new_signed_with_payer(
            &[instr],
            Some(&payer_account.pubkey()),
            &[payer_account],
            recent_blockhash,
        );

        let sig = client.send_and_confirm_transaction_with_spinner(&tx)?;

        info!("account created");
        info!("signature: {}", sig);
    }

    let account = client
        .get_account(&pubkey)
        .context("unable to get program instance account")?;

    info!("program instance account: {:?}", account);

    Ok(pubkey)
}

fn get_contract_size(_client: &RpcClient) -> Result<usize> {
    // TODO
    Ok(0)
}

static DEPLOY_PATH: &str = "target/deploy";
static PROGRAM_KEYPAIR_PATH: &str = "solana_bpf_rust_invoked-keypair.json";
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
    argument_index: &Pubkey,
    invoked_argument_index: &Pubkey,
    invoked_program_index: &Pubkey,
    invoked_program_dup_index: &Pubkey,
) -> Result<()> {
    let inst = create_instruction(&program.pubkey(), 
    argument_index, invoked_argument_index, invoked_program_index, invoked_program_dup_index)?;
    let mut tx = Transaction::new_with_payer(&[inst], Some(&config.keypair.pubkey()));
    let blockhash = client.get_recent_blockhash()?.0;
    tx.try_sign(&[&config.keypair], blockhash)?;
    let sig = client.send_and_confirm_transaction_with_spinner(&tx)?;

    info!("plant sig: {}", &sig);
    Ok(())
}

fn create_instruction(
    program_id: &Pubkey,
    argument_index: &Pubkey,
    invoked_argument_index: &Pubkey,
    invoked_program_index: &Pubkey,
    invoked_program_dup_index: &Pubkey,
) -> Result<Instruction> {
    let data: Vec<u8> = [10_u8, 11, 12, 13, 14].to_vec();
    Ok(Instruction {
        program_id: *program_id,
        accounts: vec![
            AccountMeta::new(*argument_index, true),
            AccountMeta::new(*invoked_argument_index, true),
            AccountMeta::new_readonly(*invoked_program_index, false),
            AccountMeta::new_readonly(*invoked_program_dup_index, false),
        ],
        data,
    })
}

