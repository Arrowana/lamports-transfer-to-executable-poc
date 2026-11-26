#![no_std]

use pinocchio::{
    error::ProgramError, no_allocator, nostd_panic_handler, program_entrypoint, AccountView,
    Address, ProgramResult,
};

program_entrypoint!(process_instruction);
nostd_panic_handler!();
no_allocator!();

fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [source, executable] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let amount = u64::from_le_bytes(
        instruction_data
            .try_into()
            .map_err(|_| ProgramError::InvalidInstructionData)?,
    );

    if !source.is_writable() || !executable.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    if !source.owned_by(program_id) {
        return Err(ProgramError::IllegalOwner);
    }
    if !executable.executable() {
        return Err(ProgramError::InvalidAccountData);
    }

    source.set_lamports(
        source
            .lamports()
            .checked_sub(amount)
            .ok_or(ProgramError::InsufficientFunds)?,
    );
    executable.set_lamports(
        executable
            .lamports()
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?,
    );

    Ok(())
}
