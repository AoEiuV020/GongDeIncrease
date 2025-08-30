#![allow(unexpected_cfgs)]
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // 验证账户是否可写
    if !counter_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 确保账户有足够的数据空间 (8 bytes for u64)
    if counter_account.data_len() < 8 {
        return Err(ProgramError::AccountDataTooSmall);
    }

    let instruction = instruction_data.first().copied().unwrap_or(0);

    match instruction {
        0 => {
            // 增加 counter
            let mut data = counter_account.data.borrow_mut();
            let current = u64::from_le_bytes([
                data[0], data[1], data[2], data[3], 
                data[4], data[5], data[6], data[7]
            ]);
            let new_value = current.saturating_add(1);
            let bytes = new_value.to_le_bytes();
            data[0..8].copy_from_slice(&bytes);
            msg!("Counter: {}", new_value);
        }
        1 => {
            // 关闭账户并回收租金
            let user = next_account_info(accounts_iter)?;
            
            // 验证用户是否为签名者
            if !user.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // 将账户的所有 lamports 转移给用户
            let dest_starting_lamports = user.lamports();
            **user.lamports.borrow_mut() = dest_starting_lamports
                .checked_add(counter_account.lamports())
                .ok_or(ProgramError::ArithmeticOverflow)?;
            **counter_account.lamports.borrow_mut() = 0;

            // 清空账户数据
            let mut data = counter_account.data.borrow_mut();
            data.fill(0);

            msg!("Counter 账户关闭成功，租金已返还");
        }
        _ => {
            return Err(ProgramError::InvalidInstructionData);
        }
    }

    Ok(())
}