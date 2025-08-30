#![allow(unexpected_cfgs)]
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
    system_instruction,
    program::invoke_signed,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    let instruction = instruction_data.first().copied().unwrap_or(0);

    match instruction {
        0 => {
            // 增加 counter（如果账户不存在则先创建）
            if counter_account.lamports() == 0 {
                // 账户不存在，需要创建PDA账户
                let user = next_account_info(accounts_iter)?;
                let system_program = next_account_info(accounts_iter)?;

                // 验证用户是否为签名者
                if !user.is_signer {
                    return Err(ProgramError::MissingRequiredSignature);
                }

                // 验证PDA地址
                let (expected_counter_address, bump_seed) = Pubkey::find_program_address(
                    &[b"counter", user.key.as_ref()],
                    program_id,
                );

                if counter_account.key != &expected_counter_address {
                    msg!("Counter 账户地址不正确");
                    return Err(ProgramError::InvalidAccountData);
                }

                // 创建PDA账户
                let account_space = 8; // 8 bytes for u64 counter
                let rent = Rent::get()?;
                let required_lamports = rent.minimum_balance(account_space);

                // 使用CPI创建账户
                let create_account_instruction = system_instruction::create_account(
                    user.key,
                    counter_account.key,
                    required_lamports,
                    account_space as u64,
                    program_id,
                );

                let account_infos = &[
                    user.clone(),
                    counter_account.clone(),
                    system_program.clone(),
                ];

                let seeds = &[b"counter", user.key.as_ref(), &[bump_seed]];
                let signer_seeds = &[&seeds[..]];

                invoke_signed(
                    &create_account_instruction,
                    account_infos,
                    signer_seeds,
                )?;

                // 初始化为1
                let mut data = counter_account.data.borrow_mut();
                let bytes = 1u64.to_le_bytes();
                data[0..8].copy_from_slice(&bytes);
                msg!("Counter 账户创建并初始化，值: 1");
            } else {
                // 账户已存在，增加counter
                if !counter_account.is_writable {
                    return Err(ProgramError::InvalidAccountData);
                }

                if counter_account.data_len() < 8 {
                    return Err(ProgramError::AccountDataTooSmall);
                }

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
        }
        1 => {
            // 关闭账户并回收租金
            let user = next_account_info(accounts_iter)?;
            
            // 验证账户是否可写
            if !counter_account.is_writable {
                return Err(ProgramError::InvalidAccountData);
            }
            
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