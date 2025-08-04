# solana-token22
This repository includes the basic operations of solana-spl-token-2022.

## SPL Token-2022 Dual PDA Core Functionality

This program implements a complete SPL Token-2022 lifecycle management, with a core **dual PDA authorization model**:

1.  **Token Manager (Manager PDA)**: The only PDA authorized to **create new tokens (Mint)**.
2.  **Token Vault (Vault PDA)**: The only PDA authorized to **transfer tokens**, used for securely storing and distributing tokens.

This separation of duties design greatly enhances security.

### 1. `PdaCreateTokenManager` - Create Token Manager

**Functionality**: Creates a PDA with the seed `b"token_manager"`, which will serve as the sole **Mint Authority** for all subsequent tokens.

**Client Accounts**:

```rust
vec![
    AccountMeta::new(wallet_pubkey, true),
    AccountMeta::new(PDA_TOKEN_MANAGER, false),
    AccountMeta::new_readonly(system_program::id(), false),
],
```

**On-chain Logic**:

```rust
// seeds: &[&[u8]] = &[b"token_manager"];
let instruction_pda_create_token_manager = system_instruction::create_account(...);

invoke_signed(
    &instruction_pda_create_token_manager, 
    &[payer, pda_to_create, system_program], 
    &[seeds_with_bump]
)?;
```

### 2. `PdaCreateTokenVault` - Create Token Vault

**Functionality**: Creates a PDA with the seed `b"token_vault"`. This PDA will act as the project's **token vault**. All future tokens will first be minted to an ATA owned by this vault and will be transferred out with its authorization.

**Client Accounts**:

```rust
vec![
    AccountMeta::new(wallet_pubkey, true),
    AccountMeta::new(PDA_TOKEN_VAULT, false),
    AccountMeta::new_readonly(system_program::id(), false),
],
```

**On-chain Logic**:

```rust
// seeds: &[&[u8]] = &[b"token_vault"];
let instruction_pda_create_token_vault = system_instruction::create_account(...);

invoke_signed(
    &instruction_pda_create_token_vault, 
    &[payer, pda_vault, system_program], 
    &[seeds_with_bump]
)?;
```

### 3. `TokenMint` - Create Token-2022 Token

**Functionality**: Creates a brand new SPL Token-2022 token. This process is done in two steps: first, an account is created, then it is initialized as a Mint account, and the **mint authority** is granted to the `Token Manager` PDA.

**Client Accounts**:

```rust
vec![
    AccountMeta::new(payer, true),
    AccountMeta::new(mint_account, true), // Must be a new Keypair
    AccountMeta::new(PDA_TOKEN_MANAGER, false),
    AccountMeta::new_readonly(spl_token_2022::ID, false),
    AccountMeta::new_readonly(system_program::id(), false),
    AccountMeta::new_readonly(rent::id(), false),
],
```

**On-chain Logic**:

```rust
// 1. Create account
let instruction_create_mint_account = system_instruction::create_account(...);
invoke(&instruction_create_mint_account, ...)?;

// 2. Initialize as Token-2022 Mint
let instruction_turn_into_mint_account = token_instruction::initialize_mint2(
    token_program_account.key, 
    mint_account.key,
    token_manager_account.key, // Set Mint Authority
    None, 
    6
)?;
invoke(&instruction_turn_into_mint_account, ...)?;
```

### 4. `PdaCreateAta` - Create ATA for the Vault

**Functionality**: Creates an Associated Token Account (ATA) for the `Token Vault` PDA, enabling it to hold the specified token. Since the PDA is the owner of the ATA, this operation requires the PDA's signature.

**Client Accounts**:

```rust
vec![
    AccountMeta::new(payer, true),
    AccountMeta::new(ata_to_create, false),
    AccountMeta::new(PDA_TOKEN_VAULT, false), // ATA's owner
    AccountMeta::new(mint_account, false),
    AccountMeta::new_readonly(spl_associated_token_account::id(), false),
    AccountMeta::new_readonly(spl_token_2022::ID, false),
    AccountMeta::new_readonly(system_program::id(), false),
],
```

**On-chain Logic**:

```rust
// seeds: &[&[u8]] = &[b"token_vault"];
let instruction_create_ata = ata_instruction::create_associated_token_account(...);

// Sign using the vault's seeds
invoke_signed(&instruction_create_ata, ..., &[seeds_with_bump])?;
```

### 5. `TokenMintTo` - Mint Tokens to the Vault

**Functionality**: Invokes the authority of the `Token Manager` PDA to mint a specified number of tokens into the `Token Vault`'s ATA.

**Client Accounts**:

```rust
vec![
    AccountMeta::new(payer, true),
    AccountMeta::new(mint_account, false),
    AccountMeta::new(destination_ata, false), // The vault's ATA
    AccountMeta::new(PDA_TOKEN_MANAGER, false), // The mint authority
    AccountMeta::new_readonly(spl_token_2022::ID, false),
],
```

**On-chain Logic**:

```rust
// seeds: &[&[u8]] = &[b"token_manager"];
let instruction_mint_to = token_instruction::mint_to(
    ...,
    token_manager_account.key, // Specify PDA as authority
    ...,
    amount_to_mint
)?;

// Sign using the manager's seeds
invoke_signed(&instruction_mint_to, ..., &[seeds_with_bump])?;
```

### 6. `TokenTransfer` - Transfer Tokens from the Vault

**Functionality**: Invokes the authority of the `Token Vault` PDA to transfer a specified number of tokens from its ATA to another destination ATA.

**Client Accounts**:

```rust
vec![
    AccountMeta::new(payer, true),
    AccountMeta::new(source_ata, false),      // The vault's ATA
    AccountMeta::new(destination_ata, false), // The recipient's ATA
    AccountMeta::new(PDA_TOKEN_VAULT, false), // The transfer authority
    AccountMeta::new_readonly(spl_token_2022::ID, false),
],
```

**On-chain Logic**:

```rust
// seeds: &[&[u8]] = &[b"token_vault"];
let instruction_transfer = token_instruction::transfer(
    ...,
    source_authority.key, // Specify PDA as authority
    ...,
    amount_to_transfer
)?;

// Sign using the vault's seeds
invoke_signed(&instruction_transfer, ..., &[seeds_with_bump])?;
```
