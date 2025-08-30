// ========================================
// Solana 智能合约入口 - 类似传统的"函数调度器"
// ========================================
// 在传统编程中，我们调用函数时会指定函数名和参数
// 在Solana中，所有调用都通过这个统一的入口函数，通过instruction_data来区分"函数名"

#![allow(unexpected_cfgs)]
use solana_program::{
    account_info::{next_account_info, AccountInfo}, 
    entrypoint, 
    entrypoint::ProgramResult, 
    msg, 
    program_error::ProgramError,
    pubkey::Pubkey,
};

// 引入工具模块
pub mod utils;
use utils::{
    read_gongde_value, 
    write_gongde_value, 
    validate_account_data_size, 
    GongDeInstruction,
};

// 声明这是合约的入口点 - 类似main函数
entrypoint!(process_instruction);

// 🎯 这是合约的主入口函数，所有调用都从这里开始
// 类比：这就像一个函数调度器，根据instruction_data决定调用哪个"函数"
pub fn process_instruction(
    _program_id: &Pubkey,      // 🆔 合约的唯一标识（类似类名）
    accounts: &[AccountInfo],   // 📁 函数需要操作的数据账户（类似函数参数中的对象引用）
    instruction_data: &[u8],   // 📋 调用指令和参数数据（类似函数名+参数的编码）
) -> ProgramResult {
    // 📥 从传入的账户列表中获取第一个账户（功德数据账户）
    // 类比：这就像从函数参数中取出第一个对象
    let accounts_iter = &mut accounts.iter();
    let gongde_account = next_account_info(accounts_iter)?;

    // 🔒 安全检查：确保账户可以被修改
    // 类比：检查对象是否有写权限
    if !gongde_account.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    // 📏 检查数据空间是否足够（需要4字节存储u32）
    // 类比：检查内存是否够存储数据
    validate_account_data_size(gongde_account.data_len())?;

    // 🎯 解析"函数名" - 从instruction_data解析指令类型
    // 类比：从消息中解析出要调用的函数名
    let instruction = GongDeInstruction::from_instruction_data(instruction_data)?;

    // 🚦 根据指令类型调用对应的"函数" - 这就是函数分发
    match instruction {
        GongDeInstruction::Increment => {
            // 🔢 函数名：increment() - 增加功德
            // 类比：调用 gongde.increment() 方法
            
            // 📖 读取当前的功德值（使用工具函数）
            let mut data = gongde_account.data.borrow_mut();
            let current = read_gongde_value(&data)?;
            
            // ⚠️ 检查是否已达到最大值，如果是则直接结束，不再增加
            if current == u32::MAX {
                msg!("功德圆满");
                return Ok(());
            }
            
            // ➕ 执行增加操作
            let new_value = current + 1;
            
            // 💾 将新值写回账户数据（使用工具函数）
            write_gongde_value(&mut data, new_value)?;
            
            // 📢 输出日志（类似printf或console.log）
            msg!("功德: {}", new_value);
        }
        GongDeInstruction::Close => {
            // 🗑️ 函数名：close() - 关闭账户并回收租金
            // 类比：调用 gongde.close(user) 方法
            
            // 👤 获取第二个账户参数（用户账户，接收退款）
            let user = next_account_info(accounts_iter)?;
            
            // ✍️ 验证用户是否为交易签名者（权限检查）
            // 类比：验证用户是否有删除权限
            if !user.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // 💰 将功德账户的所有租金转移给用户
            // 类比：退还押金给用户
            let dest_starting_lamports = user.lamports();
            **user.lamports.borrow_mut() = dest_starting_lamports
                .checked_add(gongde_account.lamports())
                .ok_or(ProgramError::ArithmeticOverflow)?;
            **gongde_account.lamports.borrow_mut() = 0;

            // 🧹 清空账户数据（相当于删除对象）
            let mut data = gongde_account.data.borrow_mut();
            data.fill(0);

            msg!("功德账户关闭成功，租金已返还");
        }
    }

    Ok(())
}

// ========================================
// 💡 Solana智能合约架构总结
// ========================================
// 
// 🎯 这个合约实现了两个"函数"：
// 
// 1. increment() - 指令码0
//    - 输入：一个可写的功德账户
//    - 功能：将账户中的u32值+1（如果未达到最大值）
//    - 输出：更新后的值（通过日志）
// 
// 2. close() - 指令码1  
//    - 输入：功德账户 + 用户账户
//    - 功能：删除功德账户，退还租金给用户
//    - 输出：成功消息
// 
// 🔑 核心设计特点：
//   - 单一入口：所有调用都通过process_instruction
//   - 指令分发：通过instruction_data[0]区分功能
//   - 账户模型：数据存储在accounts中，合约只处理逻辑
//   - 无状态合约：合约本身不存储数据，数据在账户中
// 
// 📊 数据流程：
//   客户端 → 创建Instruction → 打包Transaction → 发送到网络
//   网络 → 调用process_instruction → 解析指令 → 执行对应逻辑
//   合约 → 读取/写入账户数据 → 返回结果 → 客户端获取状态
// 
// 💰 经济模型：
//   - 账户需要租金（防止垃圾数据）
//   - 交易需要手续费（网络资源消耗）
//   - 账户可关闭退还租金（资源回收）
// ========================================