use anyhow::{Result};
use solana_sdk::signer::Signer;

mod solana;

/// Maximum number of bytes a program may add to an account during a single realloc
pub const MAX_PERMITTED_DATA_INCREASE: usize = 1_024 * 10;

fn main() -> Result<()> {
    env_logger::init();
    
    let config = solana::load_config()?;
    let client = solana::connect(&config)?;
    let program_keypair = solana::get_program_keypair(&client)?;
    let argument_index =
        solana::get_program_instance_account(&client, &config.keypair, &program_keypair, "testa"/*, 10, 10*/)?;
    let (invoked_argument_index, bump_seed1) =
        solana::get_program_address(&[b"You pass butter"], &program_keypair.pubkey()); 
    solana::get_specific_program_instance_account(&client, &config.keypair, &program_keypair, 42, MAX_PERMITTED_DATA_INCREASE as u64, &invoked_argument_index)?;
    let invoked_program_index =
        solana::get_program_instance_account(&client, &config.keypair, &program_keypair, "test2"/*, 10, 10*/)?;
    let invoked_program_dup_index =
        solana::get_program_instance_account(&client, &config.keypair, &program_keypair, "test3"/*, 10, 10*/)?;
    

    solana::commit(
        &config,
        &client,
        &program_keypair,
        &argument_index,
        & invoked_argument_index,
        & invoked_program_index,
        & invoked_program_dup_index,
    )?;
    Ok(())
}
