use anyhow::{Result};

mod solana;

fn main() -> Result<()> {
    env_logger::init();
    
    let config = solana::load_config()?;
    let client = solana::connect(&config)?;
    let program_keypair = solana::get_program_keypair(&client)?;
    let program_from_account =
        solana::get_program_instance_account(&client, &config.keypair, &program_keypair, "transfertest")?;
    
    let program_to_account =
        solana::get_program_instance_account(&client, &config.keypair, &program_keypair, "transferto")?;
        
    solana::commit(
        &config,
        &client,
        &program_keypair,
        &program_from_account,
        &program_to_account,
    )?;
    Ok(())
}
