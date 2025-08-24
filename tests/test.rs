// Integration test for the CPI_transfer program: verifies SPL token transfer via PDA authority.

use CPI_transfer::process_instruction;

use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    },
    solana_program_test::{processor, ProgramTest, tokio},
    solana_sdk::{signature::Signer, signer::keypair::Keypair, transaction::Transaction},
    spl_token::state::{Account, Mint},
    std::str::FromStr,
};

/// This test simulates the full flow of transferring SPL tokens
/// via a program-derived address (PDA) using a CPI.
/// It creates a mint, a source account owned by the PDA,
/// a destination account, mints tokens, invokes the program,
/// and checks that the transfer was successful.
#[tokio::test]
async fn success() {
    // The program_id must match what the CPI_transfer program expects.
    let program_id = Pubkey::from_str("TransferTokens11111111111111111111111111111").unwrap();

    // Generate keypairs for the source, mint, and destination token accounts.
    let source = Keypair::new();
    let mint = Keypair::new();
    let destination = Keypair::new();

    // Derive the PDA authority using the same seed as in the program logic.
    let (authority_pubkey, _) = Pubkey::find_program_address(&[b"authority"], &program_id);

    // Register the CPI_transfer program for testing.
    let program_test = ProgramTest::new("CPI_transfer", program_id, processor!(process_instruction));

    // Start the test validator and retrieve the test context.
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Set up test parameters.
    let amount = 10_000;    // Number of tokens to mint/transfer.
    let decimals = 9;       // Token decimals.
    let rent = Rent::default();

    // 1. Create and initialize the SPL token mint.
    let transaction = Transaction::new_signed_with_payer(
        &[
            // Create mint account.
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(Mint::LEN),
                Mint::LEN as u64,
                &spl_token::id(),
            ),
            // Initialize the mint.
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // 2. Create and initialize the source account (owned by PDA authority).
    let transaction = Transaction::new_signed_with_payer(
        &[
            // Create the source token account.
            system_instruction::create_account(
                &payer.pubkey(),
                &source.pubkey(),
                rent.minimum_balance(Account::LEN),
                Account::LEN as u64,
                &spl_token::id(),
            ),
            // Initialize the source token account, owned by the PDA.
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &source.pubkey(),
                &mint.pubkey(),
                &authority_pubkey,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &source],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // 3. Create and initialize the destination account (owned by payer).
    let transaction = Transaction::new_signed_with_payer(
        &[
            // Create the destination token account.
            system_instruction::create_account(
                &payer.pubkey(),
                &destination.pubkey(),
                rent.minimum_balance(Account::LEN),
                Account::LEN as u64,
                &spl_token::id(),
            ),
            // Initialize the destination token account.
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &destination.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &destination],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // 4. Mint tokens to the source (PDA-owned) account.
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint.pubkey(),
            &source.pubkey(),
            &payer.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // 5. Construct and send the CPI_transfer instruction, using the correct account order.
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_bincode(
            program_id,
            &(), // No instruction data required for this program.
            vec![
                AccountMeta::new(source.pubkey(), false),                // Source token account.
                AccountMeta::new_readonly(mint.pubkey(), false),         // Mint.
                AccountMeta::new(destination.pubkey(), false),           // Destination token account.
                AccountMeta::new_readonly(authority_pubkey, false),      // PDA authority.
                AccountMeta::new_readonly(spl_token::id(), false),       // SPL Token program.
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // Execute the transfer instruction.
    banks_client.process_transaction(transaction).await.unwrap();

    // 6. Fetch the destination account and verify the tokens were transferred.
    let account = banks_client
        .get_account(destination.pubkey())
        .await
        .unwrap()
        .unwrap();
    let token_account = Account::unpack(&account.data).unwrap();
    assert_eq!(token_account.amount, amount);
}
