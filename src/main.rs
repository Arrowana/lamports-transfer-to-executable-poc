use litesvm::LiteSVM;
use solana_account::Account;
use solana_address::Address;
use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::str::FromStr;

fn main() {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    let program_id = Address::new_unique();
    let source = Address::new_unique();
    let executable = Address::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();
    let source_balance = 2_000_000;
    let lamports: u64 = 1_000_000;

    svm.add_program_from_file(program_id, "target/deploy/lamport_mover.so")
        .unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();
    svm.set_account(
        source,
        Account {
            lamports: source_balance,
            data: vec![],
            owner: program_id,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let executable_before = svm.get_account(&executable).unwrap();
    assert!(executable_before.executable);

    let instruction = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(source, false),
            AccountMeta::new(executable, false),
        ],
        data: lamports.to_le_bytes().to_vec(),
    };

    let tx = Transaction::new(
        &[&payer],
        Message::new(&[instruction], Some(&payer.pubkey())),
        svm.latest_blockhash(),
    );
    svm.send_transaction(tx).unwrap();

    let source_after = svm.get_account(&source).unwrap();
    let executable_after = svm.get_account(&executable).unwrap();
    assert_eq!(source_after.lamports, source_balance - lamports);
    assert_eq!(
        executable_after.lamports,
        executable_before.lamports + lamports
    );
    println!("program moved {lamports} lamports to executable account {executable}");
}
