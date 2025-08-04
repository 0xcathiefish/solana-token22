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
const PROGRAM_ID: Pubkey = Pubkey::new_from_array([239, 147, 167, 11, 137, 173, 81, 7, 45, 177, 98, 9, 207, 233, 11, 12, 248, 90, 224, 131, 209, 182, 255, 217, 30, 208, 69, 110, 188, 40, 45, 52]);

// TODO: Fill these after PDA creation
const PDA_TOKEN_MANAGER: Pubkey = Pubkey::new_from_array([69, 182, 85, 222, 205, 145, 25, 58, 69, 10, 117, 82, 235, 87, 214, 105, 180, 62, 160, 222, 108, 124, 150, 181, 4, 116, 133, 119, 152, 240, 213, 229]);
const PDA_TOKEN_VAULT: Pubkey = Pubkey::new_from_array([58, 64, 241, 113, 23, 69, 205, 219, 50, 208, 52, 181, 39, 88, 232, 207, 122, 45, 189, 232, 156, 176, 108, 251, 202, 238, 92, 118, 37, 195, 64, 105]);

// 52BiecVgCC2kPQxyozuwfeKno5yq5DBUN3SqVXzLLyZt
const MINT_TOKEN: Pubkey = Pubkey::new_from_array([59, 188, 114, 142, 64, 167, 231, 55, 37, 59, 175, 209, 49, 189, 204, 239, 15, 240, 215, 62, 199, 247, 39, 60, 119, 125, 139, 21, 44, 9, 252, 155]);

// h3SEXeN7a9Nq5NXuMmqgSGg3u31LXf2o4wNV3ZmHJyn
const TOKEN_VAULT_ATA: Pubkey = Pubkey::new_from_array([10, 65, 252, 86, 39, 26, 140, 176, 143, 30, 244, 11, 199, 246, 205, 236, 228, 178, 122, 16, 130, 211, 115, 243, 23, 206, 78, 217, 207, 152, 209, 1]);

// 4hz8Qf1BE2jpGV5NSvkV1bq5bXCKMVLyte7VjCMsVoGR
const USER_ATA: Pubkey = Pubkey::new_from_array([55, 18, 227, 61, 221, 60, 4, 20, 107, 82, 247, 247, 30, 66, 122, 86, 149, 191, 218, 62, 210, 47, 218, 71, 110, 169, 184, 186, 89, 2, 219, 182]);


fn main() {

    dotenv().ok();
    env_logger::init();

    let client = RpcClient::new_with_commitment(RPC_DEV, CommitmentConfig::confirmed());

    info!("Load program id and rpc connect");


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
    info!("New Token Mint: {}", token_mint_account.pubkey());

    let rent = sysvar::rent::id();

    // Calculate user ATA 
    let user_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &payer.pubkey(),         // owner 
        &MINT_TOKEN,  // mint
        &spl_token_2022::ID, // Use the correct Token-2022 Program ID
    );

    info!("user_ata: {}", user_ata);

    // Calculate Vault ATA 
    let vault_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &PDA_TOKEN_VAULT,         // owner 
        &MINT_TOKEN,  // mint
        &spl_token_2022::ID, // Use the correct Token-2022 Program ID
    );
    
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
            amount_to_transfer: 500_000_000 // Transfer 500 tokens from vault to user
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
            //instruction_create_mint,              // 3. Create Token Mint
            //instruction_create_ata_for_vault,     // 4. Create ATA for Vault
            //instruction_create_ata_for_user,      // 5. Create ATA for User
            //instruction_mint_to_vault,            // 6. Mint tokens to vault
            instruction_transfer_from_vault,      // 7. Transfer tokens from vault to user
        ], 

        Some(&payer.pubkey())
    );


    let recent_blockhash = client.get_latest_blockhash().expect("Failed to get recent blockhash");
    transaction.sign(&[&payer], recent_blockhash);
    
    // 发送交易
    info!("正在调用 Token22 程序...");
    let signature = client.send_and_confirm_transaction(&transaction).expect("Failed to send transaction");
    
    info!("✅ Token22 程序调用成功！");
    info!("交易签名: {}", signature);
    info!("🔍 查看交易: https://solscan.io/tx/{}?cluster=devnet", signature);
    info!("📊 Token Mint: {}", token_mint_account.pubkey());
    info!("🏦 User ATA: {}", user_ata);
    

}