use solana_sdk::{pubkey::Pubkey,signature::{Keypair,Signer}};
use std::str::FromStr;


fn main() {


    let mut a = 3;

    match a {

        1 => {

            let pubkey = Pubkey::from_str("47zz3JfQD7FnM9QSK82VVdytmdDQd4W715zoRiTeNr6e").unwrap();
            println!("Bytes: {:?}", pubkey.to_bytes());

        },



        2 => {

            let program_id = Pubkey::from_str("9AALRRB5DfN2gNT7QmRKeQdRS5VGvZaoYBqkBQSXaAAb").unwrap();

            let seeds: &[&[u8]] = &[

                b"token_manager"
            ];

            let (pda_caculate, _) = Pubkey::find_program_address(seeds, &program_id);

            println!("Bytes: {:?}", pda_caculate.to_bytes());
            
        },

        3 => {


            let token_mint_account = Keypair::new();

            let pub1 = token_mint_account.pubkey();
            println!("New Token Mint: {}", pub1);
            println!("Bytes: {:?}", pub1.to_bytes());

        }


        _ => {
            
        }


    }

}

