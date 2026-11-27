#![no_std]

use pinocchio::{
    cpi::invoke,
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    no_allocator, nostd_panic_handler, program_entrypoint, AccountView, Address, ProgramResult,
};

program_entrypoint!(process_instruction);
nostd_panic_handler!();
no_allocator!();

fn process_instruction(
    _program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [token_account, executable_recipient, authority, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let [manual_accounting] = instruction_data else {
        return Err(ProgramError::InvalidInstructionData);
    };
    if *manual_accounting > 1 {
        return Err(ProgramError::InvalidInstructionData);
    }

    if !token_account.is_writable() || !executable_recipient.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    if !executable_recipient.executable() || !authority.is_signer() || !token_program.executable() {
        return Err(ProgramError::InvalidAccountData);
    }

    let closed_lamports = token_account.lamports();
    let close_data = [9]; // SPL Token CloseAccount
    let close_accounts = [
        InstructionAccount::writable(token_account.address()),
        InstructionAccount::writable(executable_recipient.address()),
        InstructionAccount::readonly_signer(authority.address()),
    ];
    let close = InstructionView {
        program_id: token_program.address(),
        accounts: &close_accounts,
        data: &close_data,
    };
    invoke(
        &close,
        &[&*token_account, &*executable_recipient, &*authority],
    )?;

    // The CPI updates the token account's local view, but executable accounts
    // are omitted from the update, leaving this recipient view stale.
    if *manual_accounting == 1 {
        executable_recipient.set_lamports(
            executable_recipient
                .lamports()
                .checked_add(closed_lamports)
                .ok_or(ProgramError::ArithmeticOverflow)?,
        );
    }

    Ok(())
}
