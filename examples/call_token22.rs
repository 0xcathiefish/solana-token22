#![allow(unexpected_cfgs)]
#![allow(deprecated)] 

use token22::{

    instruction::TokenInstruction
};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{Instruction,AccountMeta},
    pubkey::{Pubkey},
    signature::{Keypair, Signer},
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
    system_program,
};

use spl_token_2022;
use solana_sdk::sysvar;

use spl_associated_token_account;


use log::{info, error};
use dotenvy::dotenv;
use std::time::Duration;
use std::fs;
use serde_json;

// rpc url
const RPC_DEV: &str = "https://api.devnet.solana.com";

// TODO: Fill these after deployment
const PROGRAM_ID: Pubkey = Pubkey::new_from_array([121, 54, 71, 19, 18, 185, 110, 94, 126, 174, 70, 160, 111, 206, 57, 13, 94, 64, 99, 152, 153, 185, 145, 122, 29, 232, 183, 167, 65, 83, 127, 216]);

// TODO: Fill these after PDA creation
const PDA_TOKEN_MANAGER: Pubkey = Pubkey::new_from_array([27, 184, 188, 192, 255, 221, 163, 23, 174, 24, 116, 88, 155, 229, 136, 8, 165, 226, 110, 31, 237, 117, 237, 71, 101, 117, 233, 160, 216, 108, 254, 59]);

// PDA Token vault 
const PDA_TOKEN_VAULT: Pubkey = Pubkey::new_from_array([3, 244, 112, 7, 50, 40, 105, 99, 35, 233, 197, 192, 104, 112, 123, 247, 46, 162, 126, 85, 255, 221, 197, 38, 108, 196, 50, 97, 243, 88, 95, 194]);

// 984ryKUWtuvaRpdPkBbGdfkH8oAUNRwF2z8suEyoUKcw
//const MINT_TOKEN: Pubkey = Pubkey::new_from_array([120, 172, 237, 103, 194, 5, 24, 3, 84, 146, 224, 191, 132, 68, 241, 68, 124, 69, 127, 166, 10, 78, 171, 149, 98, 252, 27, 245, 228, 166, 82, 4]);

// 9LTJet2SbyRW2KCnLpY7WJHcN9Sp5S3b77RkZZeRSjuH
//const TOKEN_VAULT_ATA: Pubkey = Pubkey::new_from_array([123, 217, 69, 241, 227, 159, 106, 125, 198, 128, 133, 251, 192, 222, 196, 226, 138, 231, 253, 20, 95, 198, 211, 179, 105, 212, 247, 46, 222, 37, 150, 72]);

// 47zz3JfQD7FnM9QSK82VVdytmdDQd4W715zoRiTeNr6e
//const USER_ATA: Pubkey = Pubkey::new_from_array([46, 94, 27, 147, 176, 9, 143, 163, 157, 54, 120, 37, 60, 81, 214, 207, 165, 239, 252, 6, 108, 14, 107, 236, 80, 191, 114, 232, 154, 153, 132, 67]);


fn main() {

    dotenv().ok();
    env_logger::init();

    let client = RpcClient::new_with_commitment(RPC_DEV, CommitmentConfig::confirmed());

    info!("Load program id and rpc connect");

    //let USER_ATA = "9R18WG6CDnHpsnkTuKw7wiEXgijed1j9Y3w9QRnqNcJU".parse().unwrap();
    //let TOKEN_VAULT_ATA = "ECKjX2uJDbRAoaYxaqwkoAkLMagNtg3xNsUGu8FyBSsg".parse().unwrap();
    //let MINT_TOKEN = "984ryKUWtuvaRpdPkBbGdfkH8oAUNRwF2z8suEyoUKcw".parse().unwrap();

    let wallet_string = fs::read_to_string("/home/xiannvweideta/.config/solana/dev.json").unwrap();
    let keypair_bytes: Vec<u8> = serde_json::from_str(&wallet_string).unwrap();
    let payer = Keypair::try_from(&keypair_bytes[..]).unwrap();

    info!("Payer account: {}", payer.pubkey());

    let wallet_pubkey = payer.pubkey();

    match client.get_balance(&wallet_pubkey) {

        Ok(balance) => {

            if (balance as f64 / 1_000_000_000.0) < 1.0 {

                info!("request devnet airdrop...");
                let airdrop_signature = client.request_airdrop(&payer.pubkey(), 1_000_000_000).expect("Failed to get airdrop"); // 1 SOL
            
                info!("Airdrop sign : {airdrop_signature}");
            
                info!("等待空投确认...");
                loop {
                    match client.confirm_transaction(&airdrop_signature) {
                        Ok(confirmed) => {
                            if confirmed {
                                info!("✅ 空投确认成功！");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("确认空投时出错: {}", e);
                            std::thread::sleep(Duration::from_secs(2));
                            continue;
                        }
                    }
                    std::thread::sleep(Duration::from_secs(1));
                }
            }

            else {

                info!("Balance: {} SOL", balance as f64 / 1_000_000_000.0);
            }
        }


        Err(e) => {
            error!("获取余额失败: {}", e);
        }
    };

    // Generate new mint account
    let token_mint_account = Keypair::new();
    let MINT_TOKEN = token_mint_account.pubkey();
    info!("New Token Mint: {}", token_mint_account.pubkey());

    let rent = sysvar::rent::id();

    // Calculate user ATA 
    let user_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &payer.pubkey(),         // owner 
        &MINT_TOKEN,  // mint
        &spl_token_2022::ID, // Use the correct Token-2022 Program ID
    );

    let USER_ATA = user_ata;

    info!("user_ata: {}", user_ata);

    // Calculate Vault ATA 
    let vault_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &PDA_TOKEN_VAULT,         // owner 
        &MINT_TOKEN,  // mint
        &spl_token_2022::ID, // Use the correct Token-2022 Program ID
    );

    let TOKEN_VAULT_ATA = vault_ata;
    
    info!("Vault ATA: {}", vault_ata);


    // 1. Create Token Manager PDA
    let instruction_create_token_manager = Instruction::new_with_bytes(

        PROGRAM_ID, 
        &TokenInstruction::PdaCreateTokenManager.pack(), 
        vec![

            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(PDA_TOKEN_MANAGER, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // 2. Create Token Vault PDA
    let instruction_create_token_vault = Instruction::new_with_bytes(

        PROGRAM_ID, 
        &TokenInstruction::PdaCreateTokenVault.pack(), 
        vec![

            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(PDA_TOKEN_VAULT, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // 3. Create Token Mint
    let instruction_create_mint = Instruction::new_with_bytes(

        PROGRAM_ID, 
        &TokenInstruction::TokenMint.pack(), 
        vec![

            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(token_mint_account.pubkey(), true),
            AccountMeta::new(PDA_TOKEN_MANAGER, false),
            AccountMeta::new_readonly(spl_token_2022::ID, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(rent, false),
        ],
    );

    // 4. Create ATA for Vault
    let instruction_create_ata_for_vault = Instruction::new_with_bytes(

        PROGRAM_ID,
        &TokenInstruction::PdaCreateAta.pack(),
        vec![

            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(TOKEN_VAULT_ATA, false),
            AccountMeta::new(PDA_TOKEN_VAULT, false),
            AccountMeta::new(MINT_TOKEN, false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(spl_token_2022::ID, false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
    );

    // 5. Create ATA for User
    let instruction_create_ata_for_user = spl_associated_token_account::instruction::create_associated_token_account(
        &wallet_pubkey,
        &wallet_pubkey,
        &MINT_TOKEN,
        &spl_token_2022::ID,
    );

    // 6. Mint tokens to vault
    let instruction_mint_to_vault = Instruction::new_with_bytes(

        PROGRAM_ID,
        &TokenInstruction::TokenMintTo { 
            amount_to_mint: 10000_000_000 // 1000 tokens (with 6 decimals)
        }.pack(),
        vec![

            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(MINT_TOKEN, false),
            AccountMeta::new(TOKEN_VAULT_ATA, false),        // 铸造到vault的ATA
            AccountMeta::new(PDA_TOKEN_MANAGER, false),
            AccountMeta::new_readonly(spl_token_2022::ID, false),
        ],
    );

    // 7. Transfer tokens from vault to user
    let instruction_transfer_from_vault = Instruction::new_with_bytes(

        PROGRAM_ID,
        &TokenInstruction::TokenTransfer { 
            amount_to_transfer: 5000_000_000 // Transfer 500 tokens from vault to user
        }.pack(),
        vec![

            AccountMeta::new(wallet_pubkey, true),
            AccountMeta::new(TOKEN_VAULT_ATA, false),    // source_ata (vault的ATA) 
            AccountMeta::new(USER_ATA, false),           // destination_ata (用户的ATA)
            AccountMeta::new(PDA_TOKEN_VAULT, false),    // source_authority (vault PDA有签名权限)
            AccountMeta::new_readonly(spl_token_2022::ID, false),
        ],
    );

    let instruction_compute_budget_limit = ComputeBudgetInstruction::set_compute_unit_limit(300_000); // 300k CU
    let instruction_compute_budget_price = ComputeBudgetInstruction::set_compute_unit_price(100_000); // 100 microlamports per CU


    let mut transaction = Transaction::new_with_payer(

        &[
            instruction_compute_budget_limit,
            instruction_compute_budget_price,

            // Execute in sequence:
            //instruction_create_token_manager,     // 1. Create Token Manager PDA
            //instruction_create_token_vault,       // 2. Create Token Vault PDA  
            instruction_create_mint,              // 3. Create Token Mint
            instruction_create_ata_for_vault,     // 4. Create ATA for Vault
            instruction_create_ata_for_user,      // 5. Create ATA for User
            instruction_mint_to_vault,            // 6. Mint tokens to vault
            instruction_transfer_from_vault,      // 7. Transfer tokens from vault to user
        ], 

        Some(&payer.pubkey())
    );


    let recent_blockhash = client.get_latest_blockhash().expect("Failed to get recent blockhash");
    transaction.sign(&[&payer,&token_mint_account], recent_blockhash);
    
    // 发送交易
    info!("正在调用 Token22 程序...");
    let signature = client.send_and_confirm_transaction(&transaction).expect("Failed to send transaction");
    
    info!("✅ Token22 程序调用成功！");
    info!("交易签名: {}", signature);
    info!("🔍 查看交易: https://solscan.io/tx/{}?cluster=devnet", signature);
    info!("📊 Token Mint: {}", token_mint_account.pubkey());
    info!("🏦 User ATA: {}", user_ata);
    

}