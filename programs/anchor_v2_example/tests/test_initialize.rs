
use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
};


use anchor_lang::prelude::msg;
use anchor_lang::solana_program;
use litesvm_token::CreateMint;
use litesvm_token::spl_token;
use anchor_lang::prelude::Pubkey;

use litesvm_token::MintTo;

#[test]
fn test_initialize() {
    let program_id = anchor_v2_example::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_v2_example.so");
    svm.add_program(program_id, bytes).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    
    let instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::Initialize {}.data(),
        anchor_v2_example::accounts::Initialize {}.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());
}

#[test]
fn test_init_sender_token_account() {
    let program_id = anchor_v2_example::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_v2_example.so");
    svm.add_program(program_id, bytes).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let mint = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let (token_account, _bump) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), mint.as_ref()],
        &program_id,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::InitTokenAccount {}.data(),
        anchor_v2_example::accounts::InitTokenAccount {
            signer: payer.pubkey(),
            token_account,
            mint,
            system_program: solana_program::system_program::id(),
            token_program: spl_token::id(),
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);

    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    println!("Token account initialized successfully");

    println!("Minting tokens to senders token account!");

    MintTo::new(&mut svm, &payer, &mint, &token_account, 10000)
        .send()
        .unwrap();

    msg!("Tokens minted successfully");

    let receiver = Keypair::new();
    svm.airdrop(&receiver.pubkey(), 1_000_000_000).unwrap();

    let (receiver_token_account, _bump) = Pubkey::find_program_address(
        &[b"token-account", receiver.pubkey().as_ref(), mint.as_ref()],
        &program_id,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::InitTokenAccount {}.data(),
        anchor_v2_example::accounts::InitTokenAccount {
            signer: receiver.pubkey(),
            token_account: receiver_token_account,
            mint,
            system_program: solana_program::system_program::id(),
            token_program: spl_token::id(),
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&receiver.pubkey()), &blockhash);

    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&receiver]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    println!("Receiver Token account initialized successfully");

    let transfer_tokens_instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::TransferTokens { amount: 10000 }.data(),
        anchor_v2_example::accounts::TransferToken {
            sender: payer.pubkey(),
            receiver: receiver.pubkey(),
            sender_token_account: token_account,
            recipient_token_account: receiver_token_account,
            mint,
            token_program: spl_token::id(),
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[transfer_tokens_instruction], Some(&payer.pubkey()), &blockhash);

    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    println!("Tokens transferred successfully");

    let close_sender_token_account_instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::CloseTokenAccount {}.data(),
        anchor_v2_example::accounts::CloseTokenAccount {
            signer: payer.pubkey(),
            token_account,
            mint,
            system_program: solana_program::system_program::id(),
            token_program: spl_token::id(),
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[close_sender_token_account_instruction], Some(&payer.pubkey()), &blockhash);

    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    println!("Sender token account closed successfully");
}


#[test]
fn test_init_pool() {
    let program_id = anchor_v2_example::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_v2_example.so");
    svm.add_program(program_id, bytes).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let (mint_authority_pda, _bump) = Pubkey::find_program_address(&[b"mint_authority".as_ref()], &program_id);

    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let pool_account = Pubkey::find_program_address(
        &[b"pool".as_ref(), mint_a.as_ref(), mint_b.as_ref()],
        &program_id,
    ).0;

    let lp_mint = Pubkey::find_program_address(
        &[b"lp_mint".as_ref(), pool_account.as_ref()],
        &program_id,
    ).0;

    let token_a_vault = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), pool_account.as_ref(), mint_a.as_ref()],
        &program_id,
    ).0;

    let token_b_vault = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), pool_account.as_ref(), mint_b.as_ref()],
        &program_id,
    ).0;

    let initialize_pool_instruction = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::InitPool { pool_name: "Test Pool".to_string() }.data(),
        anchor_v2_example::accounts::InitPool {
            signer: payer.pubkey(),
            mint_authority: mint_authority_pda,
            mint_a,
            mint_b,
            pool: pool_account,
            token_a_vault,
            token_b_vault,
            lp_mint,
            system_program: solana_program::system_program::id(),
            token_program: spl_token::id(),
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[initialize_pool_instruction], Some(&payer.pubkey()), &blockhash);

    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    msg!("Pool initialized successfully");
}

#[test]
fn test_add_liquidity() {
    let program_id = anchor_v2_example::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_v2_example.so");
    svm.add_program(program_id, bytes).unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 5_000_000_000).unwrap();

    // ---------- mints + pool PDAs ----------
    let (mint_authority_pda, _) =
        Pubkey::find_program_address(&[b"mint_authority".as_ref()], &program_id);

    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let pool = Pubkey::find_program_address(
        &[b"pool".as_ref(), mint_a.as_ref(), mint_b.as_ref()],
        &program_id,
    ).0;

    let lp_mint = Pubkey::find_program_address(
        &[b"lp_mint".as_ref(), pool.as_ref()],
        &program_id,
    ).0;

    let token_a_vault = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), pool.as_ref(), mint_a.as_ref()],
        &program_id,
    ).0;

    let token_b_vault = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), pool.as_ref(), mint_b.as_ref()],
        &program_id,
    ).0;

    // ---------- init pool ----------
    let init_pool_ix = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::InitPool {
            pool_name: "Test Pool".to_string(),
        }
        .data(),
        anchor_v2_example::accounts::InitPool {
            signer: payer.pubkey(),
            mint_authority: mint_authority_pda,
            mint_a,
            mint_b,
            pool,
            token_a_vault,
            token_b_vault,
            lp_mint,
            system_program: solana_program::system_program::id(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[init_pool_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    // ---------- provider's three token accounts (mint_a, mint_b, lp_mint) ----------
    let (provider_ata_a, _) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), mint_a.as_ref()],
        &program_id,
    );
    let (provider_ata_b, _) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), mint_b.as_ref()],
        &program_id,
    );
    let (provider_ata_lp, _) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), lp_mint.as_ref()],
        &program_id,
    );

    for (ata, mint) in [
        (provider_ata_a, mint_a),
        (provider_ata_b, mint_b),
        (provider_ata_lp, lp_mint),
    ] {
        let ix = Instruction::new_with_bytes(
            program_id,
            &anchor_v2_example::instruction::InitTokenAccount {}.data(),
            anchor_v2_example::accounts::InitTokenAccount {
                signer: payer.pubkey(),
                token_account: ata,
                mint,
                system_program: solana_program::system_program::id(),
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
        );
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
        svm.send_transaction(tx).unwrap();
    }

    // ---------- seed provider with tokens ----------
    MintTo::new(&mut svm, &payer, &mint_a, &provider_ata_a, 1_000_000)
        .send()
        .unwrap();
    MintTo::new(&mut svm, &payer, &mint_b, &provider_ata_b, 2_000_000)
        .send()
        .unwrap();

    // ---------- add_liquidity ----------
    let add_liquidity_ix = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::AddLiquidity {
            amount_a: 500_000,
            amount_b: 1_000_000,
        }
        .data(),
        anchor_v2_example::accounts::AddLiquidity {
            provider: payer.pubkey(),
            mint_a,
            mint_b,
            mint_authority: mint_authority_pda,
            pool,
            token_a_vault,
            token_b_vault,
            lp_mint,
            provider_token_a_account: provider_ata_a,
            provider_token_b_account: provider_ata_b,
            provider_lp_token_account: provider_ata_lp,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[add_liquidity_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    println!("Liquidity added successfully");
}

#[test]
fn test_remove_liquidity() {
    let program_id = anchor_v2_example::id();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/anchor_v2_example.so");
    svm.add_program(program_id, bytes).unwrap();

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 5_000_000_000).unwrap();

    // ---------- mints + pool PDAs ----------
    let (mint_authority_pda, _) =
        Pubkey::find_program_address(&[b"mint_authority".as_ref()], &program_id);

    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .freeze_authority(&payer.pubkey())
        .token_program_id(&spl_token::ID)
        .send()
        .unwrap();

    let pool = Pubkey::find_program_address(
        &[b"pool".as_ref(), mint_a.as_ref(), mint_b.as_ref()],
        &program_id,
    ).0;

    let lp_mint = Pubkey::find_program_address(
        &[b"lp_mint".as_ref(), pool.as_ref()],
        &program_id,
    ).0;

    let token_a_vault = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), pool.as_ref(), mint_a.as_ref()],
        &program_id,
    ).0;

    let token_b_vault = Pubkey::find_program_address(
        &[b"token_vault".as_ref(), pool.as_ref(), mint_b.as_ref()],
        &program_id,
    ).0;

    // ---------- init pool ----------
    let init_pool_ix = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::InitPool {
            pool_name: "Test Pool".to_string(),
        }
        .data(),
        anchor_v2_example::accounts::InitPool {
            signer: payer.pubkey(),
            mint_authority: mint_authority_pda,
            mint_a,
            mint_b,
            pool,
            token_a_vault,
            token_b_vault,
            lp_mint,
            system_program: solana_program::system_program::id(),
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let m = Message::new_with_blockhash(&[init_pool_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(m), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    // ---------- provider token accounts ----------
    let (provider_ata_a, _) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), mint_a.as_ref()],
        &program_id,
    );
    let (provider_ata_b, _) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), mint_b.as_ref()],
        &program_id,
    );
    let (provider_ata_lp, _) = Pubkey::find_program_address(
        &[b"token-account", payer.pubkey().as_ref(), lp_mint.as_ref()],
        &program_id,
    );

    for (ata, mint) in [
        (provider_ata_a, mint_a),
        (provider_ata_b, mint_b),
        (provider_ata_lp, lp_mint),
    ] {
        let ix = Instruction::new_with_bytes(
            program_id,
            &anchor_v2_example::instruction::InitTokenAccount {}.data(),
            anchor_v2_example::accounts::InitTokenAccount {
                signer: payer.pubkey(),
                token_account: ata,
                mint,
                system_program: solana_program::system_program::id(),
                token_program: spl_token::id(),
            }
            .to_account_metas(None),
        );
        let blockhash = svm.latest_blockhash();
        let m = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(m), &[&payer]).unwrap();
        svm.send_transaction(tx).unwrap();
    }

    // ---------- seed provider ----------
    MintTo::new(&mut svm, &payer, &mint_a, &provider_ata_a, 1_000_000)
        .send()
        .unwrap();
    MintTo::new(&mut svm, &payer, &mint_b, &provider_ata_b, 2_000_000)
        .send()
        .unwrap();

    // ---------- add_liquidity ----------
    let add_liquidity_ix = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::AddLiquidity {
            amount_a: 500_000,
            amount_b: 1_000_000,
        }
        .data(),
        anchor_v2_example::accounts::AddLiquidity {
            provider: payer.pubkey(),
            mint_a,
            mint_b,
            mint_authority: mint_authority_pda,
            pool,
            token_a_vault,
            token_b_vault,
            lp_mint,
            provider_token_a_account: provider_ata_a,
            provider_token_b_account: provider_ata_b,
            provider_lp_token_account: provider_ata_lp,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let m = Message::new_with_blockhash(&[add_liquidity_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(m), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    println!("Liquidity added; now removing half of it.");

    // First deposit minted sqrt(500_000 * 1_000_000) ≈ 707_106 LP.
    // Burn half (≈ 353_553) — expect ~half of each reserve back.
    let lp_to_burn: u64 = 353_553;

    let remove_liquidity_ix = Instruction::new_with_bytes(
        program_id,
        &anchor_v2_example::instruction::RemoveLiquidity { lp_amount: lp_to_burn }.data(),
        anchor_v2_example::accounts::RemoveLiquidity {
            provider: payer.pubkey(),
            mint_a,
            mint_b,
            mint_authority: mint_authority_pda,
            pool,
            token_a_vault,
            token_b_vault,
            lp_mint,
            provider_token_a_account: provider_ata_a,
            provider_token_b_account: provider_ata_b,
            provider_lp_token_account: provider_ata_lp,
            token_program: spl_token::id(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let m = Message::new_with_blockhash(&[remove_liquidity_ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(m), &[&payer]).unwrap();
    let res = svm.send_transaction(tx).unwrap();
    println!("logs: {:#?}", res.logs);

    println!("Liquidity removed successfully");
}