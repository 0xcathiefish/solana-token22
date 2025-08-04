use borsh::{BorshSerialize,BorshDeserialize};

// Token22 instruction

#[derive(BorshSerialize,BorshDeserialize)]
pub enum TokenInstruction {

    PdaCreateTokenManager,
    PdaCreateTokenVault,
    PdaCreateAta,
    TokenMint,

    TokenMintTo {

        amount_to_mint: u64,
    },

    TokenTransfer {

        amount_to_transfer: u64,
    }

}

impl TokenInstruction {

    // pack
    pub fn pack(self) -> Vec<u8> {

        self.try_to_vec().expect("Failed to serilize")
    }

    // unpack
    pub fn unpack(instruction_data: &[u8]) -> Result<Self,borsh::maybestd::io::Error> {

        Self::try_from_slice(instruction_data)
    }
}