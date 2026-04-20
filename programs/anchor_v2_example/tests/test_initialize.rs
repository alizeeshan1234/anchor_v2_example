
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