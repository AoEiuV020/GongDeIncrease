#![allow(unexpected_cfgs)]
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};

entrypoint!(process_instruction);

/// Counter 账户的数据结构
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}

/// 指令类型
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// 初始化 counter 账户
    Initialize,
    /// 增加 counter 值
    Increment,
    /// 重置 counter 值
    Reset,
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = CounterInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        CounterInstruction::Initialize => {
            msg!("指令: 初始化 Counter");
            process_initialize(program_id, accounts)
        }
        CounterInstruction::Increment => {
            msg!("指令: 增加 Counter");
            process_increment(accounts)
        }
        CounterInstruction::Reset => {
            msg!("指令: 重置 Counter");
            process_reset(accounts)
        }
    }
}

fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;

    // 验证账户所有者
    if counter_account.owner != program_id {
        msg!("Counter 账户不属于此程序");
        return Err(ProgramError::IncorrectProgramId);
    }

    // 验证账户是否可写
    if !counter_account.is_writable {
        msg!("Counter 账户不可写");
        return Err(ProgramError::InvalidAccountData);
    }

    // 验证用户是否为签名者
    if !user.is_signer {
        msg!("用户未签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data = CounterAccount::try_from_slice(&counter_account.data.borrow())?;
    counter_data.count = 0;

    counter_data.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("Counter 初始化成功，初始值: {}", counter_data.count);

    Ok(())
}

fn process_increment(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // 验证账户是否可写
    if !counter_account.is_writable {
        msg!("Counter 账户不可写");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut counter_data = CounterAccount::try_from_slice(&counter_account.data.borrow())?;
    counter_data.count = counter_data.count.checked_add(1).unwrap();

    counter_data.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("Counter 增加成功，当前值: {}", counter_data.count);

    Ok(())
}

fn process_reset(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;

    // 验证账户是否可写
    if !counter_account.is_writable {
        msg!("Counter 账户不可写");
        return Err(ProgramError::InvalidAccountData);
    }

    // 验证用户是否为签名者
    if !user.is_signer {
        msg!("用户未签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut counter_data = CounterAccount::try_from_slice(&counter_account.data.borrow())?;
    counter_data.count = 0;

    counter_data.serialize(&mut &mut counter_account.data.borrow_mut()[..])?;
    msg!("Counter 重置成功，当前值: {}", counter_data.count);

    Ok(())
}