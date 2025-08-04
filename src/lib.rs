#![allow(unexpected_cfgs)]
#![allow(deprecated)] 

pub mod instruction;
use crate::instruction::{TokenInstruction};

mod check;
use crate::check::Check;


use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint::{ProgramResult}, 
    entrypoint,
    msg, 
    program::{invoke, invoke_signed}, 
    program_pack::Pack, 
    pubkey::Pubkey, 
    system_instruction, 
    sysvar::{rent::Rent, Sysvar} 
};

use spl_token_2022::{

    instruction as token_instruction,
};

use spl_associated_token_account::{

    instruction as ata_instruction,
};

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],

) -> ProgramResult {

    Check::check_instr(instruction_data)?;

    let instruction = TokenInstruction::unpack(instruction_data)?;

    match instruction {

        TokenInstruction::PdaCreateTokenManager => {

            msg!("Instruction:PdaCreateTokenManager");
            pda_create_token_manager(program_id,accounts)?;
        },

        TokenInstruction::PdaCreateTokenVault => {

            msg!("Instruction: PdaCreateTokenVault");
            pda_create_token_vault(program_id,accounts)?;
        },

        TokenInstruction::TokenMint => {

            msg!("Instruction: TokenMint");
            token_mint(program_id,accounts)?;
        },

        TokenInstruction::PdaCreateAta => {

            msg!("Instruction: PdaCreateAta");
            pda_create_ata(program_id,accounts)?;
        }


        TokenInstruction::TokenMintTo {amount_to_mint} => {

            msg!("Instruction: TokenMintTo");
            token_mint_to(program_id,accounts,amount_to_mint)?;
        },

        TokenInstruction::TokenTransfer {amount_to_transfer} => {

            msg!("Instruction: TokenTransfer");
            token_transfer(program_id,accounts,amount_to_transfer)?;
        },
    }

    Ok(())
}



pub fn pda_create_token_manager(program_id: &Pubkey ,accounts: &[AccountInfo]) -> ProgramResult {


    let accounts_iter = &mut accounts.iter();
    let payer_account               = next_account_info(accounts_iter)?;
    let pda_to_create               = next_account_info(accounts_iter)?;
    let system_program_account      = next_account_info(accounts_iter)?;

    let seeds: &[&[u8]] = &[b"token_manager"];
    let (_, seeds_bump) = Pubkey::find_program_address(seeds, program_id);
    let seeds_with_bump: &[&[u8]] = &[b"token_manager",&[seeds_bump]];
    
    Check::check_whitelist(payer_account)?;
    Check::check_token_manager(pda_to_create, program_id)?;
    Check::check_system_program(system_program_account)?;

    let instruction_pda_create_token_manager = system_instruction::create_account(

        payer_account.key, 
        pda_to_create.key, 
        Rent::get()?.minimum_balance(0), 
        0, 
        program_id
    );

    invoke_signed(
        
        &instruction_pda_create_token_manager, 
        &[

            payer_account.clone(),
            pda_to_create.clone(),
            system_program_account.clone(),
        ], 
        &[seeds_with_bump]
    )?;

    Ok(())
}


pub fn pda_create_token_vault(program_id: &Pubkey ,accounts: &[AccountInfo]) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let payer_account               = next_account_info(accounts_iter)?;
    let pda_vault                   = next_account_info(accounts_iter)?;
    let system_program_account      = next_account_info(accounts_iter)?;

    let seeds: &[&[u8]] = &[b"token_vault"];
    let (_, seeds_bump) = Pubkey::find_program_address(seeds, program_id);
    let seeds_with_bump: &[&[u8]] = &[b"token_vault",&[seeds_bump]];
    
    Check::check_whitelist(payer_account)?;
    Check::check_pda_vault(pda_vault, program_id)?;
    Check::check_system_program(system_program_account)?;

    let instruction_pda_create_token_vault = system_instruction::create_account(

        payer_account.key, 
        pda_vault.key, 
        Rent::get()?.minimum_balance(0), 
        0, 
        program_id
    );

    invoke_signed(
        
        &instruction_pda_create_token_vault, 
        &[

            payer_account.clone(),
            pda_vault.clone(),
            system_program_account.clone(),
        ], 
        &[seeds_with_bump]
    )?;

    Ok(())
}



pub fn token_mint(program_id: &Pubkey ,accounts: &[AccountInfo]) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let payer_account               = next_account_info(accounts_iter)?;
    let mint_account                = next_account_info(accounts_iter)?;
    let token_manager_account       = next_account_info(accounts_iter)?;
    let token_program_account       = next_account_info(accounts_iter)?;
    let system_program_account      = next_account_info(accounts_iter)?;
    let rent_sysvar                 = next_account_info(accounts_iter)?;

    Check::check_whitelist(payer_account)?;
    Check::check_system_program(system_program_account)?;
    Check::check_token_program(token_program_account)?;

    let seeds: &[&[u8]] = &[b"token_manager"];
    let (_, seeds_bump) = Pubkey::find_program_address(seeds, program_id);
    let seeds_with_bump: &[&[u8]] = &[b"token_manager",&[seeds_bump]];

    // First, create a normal accunt itself using system instruction

    let instruction_create_mint_account = system_instruction::create_account(
        
        payer_account.key, 
        mint_account.key, 
        Rent::get()?.minimum_balance(spl_token_2022::state::Mint::LEN), 
        spl_token_2022::state::Mint::LEN as u64, 
        token_program_account.key,
    );

    invoke(
        
        &instruction_create_mint_account, 
        &[

            payer_account.clone(),
            mint_account.clone(),
            system_program_account.clone(),
        ], 
    )?;


    // Next, initialize the mint account into token-22 account using token22 program

    let instruction_turn_into_mint_account = token_instruction::initialize_mint2(
        
        token_program_account.key, 
        mint_account.key,
        token_manager_account.key, 
        None, 
        6
    )?;

    invoke_signed(
        
        &instruction_turn_into_mint_account, 
        &[
            mint_account.clone(),
            rent_sysvar.clone(),
        ], 
        &[seeds_with_bump]
    )?;

    Ok(())
}



pub fn pda_create_ata(program_id: &Pubkey ,accounts: &[AccountInfo]) -> ProgramResult {

    let accounts_iter     = &mut accounts.iter();
    let payer_account                   = next_account_info(accounts_iter)?;
    let ata_to_create                   = next_account_info(accounts_iter)?;
    let pda_vault                       = next_account_info(accounts_iter)?;
    let mint_account                    = next_account_info(accounts_iter)?;
    let ata_program_account             = next_account_info(accounts_iter)?;
    let token_program_account           = next_account_info(accounts_iter)?;
    let system_program_account          = next_account_info(accounts_iter)?;

    let seeds: &[&[u8]] = &[b"token_vault"];
    let (_, seeds_bump) = Pubkey::find_program_address(seeds, program_id);
    let seeds_with_bump: &[&[u8]] = &[b"token_vault",&[seeds_bump]];
    
    Check::check_whitelist(payer_account)?;
    Check::check_token_program(token_program_account)?;
    Check::check_system_program(system_program_account)?;

    let instruction_create_ata = ata_instruction::create_associated_token_account(

        payer_account.key, 
        pda_vault.key, 
        mint_account.key, 
        token_program_account.key
    );

    invoke_signed(
        
        &instruction_create_ata, 
        &[

            payer_account.clone(),
            ata_to_create.clone(),
            pda_vault.clone(),
            mint_account.clone(),
            system_program_account.clone(),
            token_program_account.clone(),
            ata_program_account.clone(),
        ],
        &[seeds_with_bump]
    )?;

    Ok(())
}


pub fn token_mint_to(program_id: &Pubkey ,accounts: &[AccountInfo],amount_to_mint: u64) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let payer_account               = next_account_info(accounts_iter)?;
    let mint_account                = next_account_info(accounts_iter)?;
    let destination_ata             = next_account_info(accounts_iter)?;
    let token_manager_account       = next_account_info(accounts_iter)?;
    let token_program_account       = next_account_info(accounts_iter)?;

    let seeds: &[&[u8]] = &[b"token_manager"];
    let (_, seeds_bump) = Pubkey::find_program_address(seeds, program_id);
    let seeds_with_bump: &[&[u8]] = &[b"token_manager",&[seeds_bump]];
    
    Check::check_whitelist(payer_account)?;
    Check::check_token_manager(token_manager_account, program_id)?;
    Check::check_token_program(token_program_account)?;

    let instruction_mint_to = token_instruction::mint_to(

        token_program_account.key,
        mint_account.key,
        destination_ata.key,
        token_manager_account.key,
        &[],
        amount_to_mint // Amount to mint (adjust as needed)
    )?;

    invoke_signed(
        
        &instruction_mint_to, 
        &[

            mint_account.clone(),
            destination_ata.clone(),
            token_manager_account.clone(),
            token_program_account.clone(),
        ], 
        &[seeds_with_bump]
    )?;

    Ok(())
}


pub fn token_transfer(program_id: &Pubkey ,accounts: &[AccountInfo],amount_to_transfer: u64) -> ProgramResult{

    let accounts_iter = &mut accounts.iter();
    let payer_account               = next_account_info(accounts_iter)?;
    let source_ata                  = next_account_info(accounts_iter)?;
    let destination_ata             = next_account_info(accounts_iter)?;
    let source_authority            = next_account_info(accounts_iter)?;
    let token_program_account       = next_account_info(accounts_iter)?;

    let seeds: &[&[u8]] = &[b"token_vault"];
    let (_, seeds_bump) = Pubkey::find_program_address(seeds, program_id);
    let seeds_with_bump: &[&[u8]] = &[b"token_vault",&[seeds_bump]];
    
    Check::check_whitelist(payer_account)?;
    Check::check_token_program(token_program_account)?;

    let instruction_transfer = token_instruction::transfer(

        token_program_account.key,
        source_ata.key,
        destination_ata.key,
        source_authority.key,
        &[],
        amount_to_transfer
    )?;

    invoke_signed(
        
        &instruction_transfer, 
        &[

            source_ata.clone(),
            destination_ata.clone(),
            source_authority.clone(),
            token_program_account.clone(),
        ], 
        &[seeds_with_bump]
    )?;

    Ok(())
}
