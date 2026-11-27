use litesvm::LiteSVM;
use solana_account::Account;
use solana_address::Address;
use solana_instruction::{error::InstructionError, AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_program_pack::Pack;
use solana_signer::Signer;
use solana_transaction::Transaction;
use solana_transaction_error::TransactionError;
use spl_token_interface::state::{Account as TokenAccount, AccountState};

const TOKEN_ACCOUNT_LAMPORTS: u64 = 2_000_000;
const PROGRAM_PATH: &str = "target/deploy/lamport_mover.so";

fn close_through_cpi(manual_accounting: bool) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();
    let caller_program = Address::new_unique();
    let executable_recipient = Address::new_unique();
    let token_account = Address::new_unique();

    svm.add_program_from_file(caller_program, PROGRAM_PATH)
        .unwrap();
    // A second loaded program is the executable close recipient. It is passed
    // as data to the caller, but is not itself invoked.
    svm.add_program_from_file(executable_recipient, PROGRAM_PATH)
        .unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let mut token_data = vec![0; TokenAccount::LEN];
    TokenAccount::pack(
        TokenAccount {
            mint: Address::new_unique(),
            owner: payer.pubkey(),
            state: AccountState::Initialized,
            ..TokenAccount::default()
        },
        &mut token_data,
    )
    .unwrap();
    svm.set_account(
        token_account,
        Account {
            lamports: TOKEN_ACCOUNT_LAMPORTS,
            data: token_data,
            owner: spl_token_interface::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();

    let recipient_before = svm.get_account(&executable_recipient).unwrap();
    assert!(recipient_before.executable);

    let instruction = Instruction {
        program_id: caller_program,
        accounts: vec![
            AccountMeta::new(token_account, false),
            AccountMeta::new(executable_recipient, false),
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(spl_token_interface::ID, false),
        ],
        data: vec![u8::from(manual_accounting)],
    };
    let tx = Transaction::new(
        &[&payer],
        Message::new(&[instruction], Some(&payer.pubkey())),
        svm.latest_blockhash(),
    );
    let result = svm.send_transaction(tx);

    if manual_accounting {
        result.unwrap();
        assert!(svm.get_account(&token_account).is_none());
        assert_eq!(
            svm.get_account(&executable_recipient).unwrap().lamports,
            recipient_before.lamports + TOKEN_ACCOUNT_LAMPORTS
        );
    } else {
        let error = result.unwrap_err();
        assert_eq!(
            error.err,
            TransactionError::InstructionError(0, InstructionError::ExternalAccountLamportSpend)
        );
        // The failed transaction is atomic.
        assert_eq!(
            svm.get_account(&token_account).unwrap().lamports,
            TOKEN_ACCOUNT_LAMPORTS
        );
        assert_eq!(
            svm.get_account(&executable_recipient).unwrap().lamports,
            recipient_before.lamports
        );
    }
}

#[test]
fn cpi_close_to_executable_without_fix_fails() {
    close_through_cpi(false);
}

#[test]
fn cpi_close_to_executable_with_manual_accounting_succeeds() {
    close_through_cpi(true);
}
