use {
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
    },
    spl_token::{
        instruction::transfer_checked,
        state::{Account, Mint},
    },
};

/// Entrypoint for the Solana program. This macro designates `process_instruction` as the main entrypoint.
solana_program::entrypoint!(process_instruction);

/// Processes an instruction to transfer SPL tokens using a program-derived address (PDA) as authority.
///
/// # Parameters
/// - `program_id`: The public key of the program.
/// - `accounts`: The accounts required for the transfer, in order:
///     1. Source token account (must be owned by the PDA authority)
///     2. Mint account
///     3. Destination token account
///     4. PDA authority account (must match PDA derived from seeds)
///     5. SPL Token program account
/// - `_instruction_data`: Instruction data (unused in this implementation).
///
/// # Returns
/// - `ProgramResult`: Ok(()) on success, or an error value on failure.
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // Create an iterator over the provided accounts.
    let account_info_iter = &mut accounts.iter();

    // 1. Source SPL token account (owned by the PDA authority).
    let source_info = next_account_info(account_info_iter)?;
    // 2. The Mint account for the SPL token.
    let mint_info = next_account_info(account_info_iter)?;
    // 3. Destination SPL token account (will receive tokens).
    let destination_info = next_account_info(account_info_iter)?;
    // 4. PDA authority account (must match derived PDA).
    let authority_info = next_account_info(account_info_iter)?;
    // 5. SPL token program account.
    let token_program_info = next_account_info(account_info_iter)?;

    // Derive the expected PDA authority using the seed "authority" and the program_id.
    let (expected_authority, bump_seed) = Pubkey::find_program_address(&[b"authority"], program_id);

    // Ensure the provided authority account matches the derived PDA.
    if expected_authority != *authority_info.key {
        msg!("Invalid PDA authority provided.");
        return Err(ProgramError::InvalidSeeds);
    }

    // Unpack the source account data to get the token amount.
    let _source_account = Account::unpack(&source_info.try_borrow_data()?)?;
    let amount: u64 = 10_000;

    // Unpack the mint account to get the token decimals.
    let mint = Mint::unpack(&mint_info.try_borrow_data()?)?;
    let decimals = mint.decimals;

    // Prepare the PDA authority seeds for signature.
    let authority_seeds: &[&[u8]] = &[b"authority", &[bump_seed]];

    // Log the transfer attempt.
    msg!(
        "Transferring {} tokens (decimals: {}) from {} to {} using PDA {}",
        amount,
        decimals,
        source_info.key,
        destination_info.key,
        authority_info.key,
    );

    // Construct and invoke the SPL token transfer_checked instruction.
    invoke_signed(
        &transfer_checked(
            token_program_info.key,
            source_info.key,
            mint_info.key,
            destination_info.key,
            authority_info.key,
            &[],
            amount,
            decimals,
        )?,
        &[
            source_info.clone(),
            mint_info.clone(),
            destination_info.clone(),
            authority_info.clone(),
            token_program_info.clone(),
        ],
        &[authority_seeds],
    )?;

    // Indicate success.
    msg!("Transfer complete.");
    Ok(())
}
