#![allow(unexpected_cfgs)]
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    program::invoke_signed,
    rent::Rent,
    sysvar::Sysvar,
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
    /// 关闭账户并取回租金
    Close,
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
        CounterInstruction::Close => {
            msg!("指令: 关闭 Counter 账户");
            process_close(accounts)
        }
    }
}

fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let user = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // 验证用户是否为签名者
    if !user.is_signer {
        msg!("用户未签名");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // 验证 PDA 地址
    let (expected_counter_address, bump_seed) = Pubkey::find_program_address(
        &[b"counter", user.key.as_ref()],
        program_id,
    );

    if counter_account.key != &expected_counter_address {
        msg!("Counter 账户地址不正确");
        return Err(ProgramError::InvalidAccountData);
    }

    // 如果账户已经初始化，直接返回
    if counter_account.lamports() > 0 {
        msg!("Counter 账户已存在");
        return Ok(());
    }

    // 创建 PDA 账户
    let account_space = std::mem::size_of::<CounterAccount>();
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(account_space);

    // 使用 CPI 创建账户
    let create_account_instruction = solana_program::system_instruction::create_account(
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

    solana_program::program::invoke_signed(
        &create_account_instruction,
        account_infos,
        signer_seeds,
    )?;

    // 初始化账户数据
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

fn process_close(accounts: &[AccountInfo]) -> ProgramResult {
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

    // 将账户的所有 lamports 转移给用户
    let dest_starting_lamports = user.lamports();
    **user.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(counter_account.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **counter_account.lamports.borrow_mut() = 0;

    // 清空账户数据
    let mut data = counter_account.data.borrow_mut();
    data.fill(0);

    msg!("Counter 账户关闭成功，租金已返还给用户");
    Ok(())
}