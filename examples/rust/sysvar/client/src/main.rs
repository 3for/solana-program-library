use anyhow::{Result};

mod solana;

fn main() -> Result<()> {
    env_logger::init();
    
    let config = solana::load_config()?;
    let client = solana::connect(&config)?;
    let program_keypair = solana::get_program_keypair(&client)?;
          
    solana::commit(
        &config,
        &client,
        &program_keypair,
    )?;
    Ok(())
}
