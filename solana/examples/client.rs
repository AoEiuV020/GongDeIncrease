// ========================================
// Solana Counter 智能合约客户端
// 这个客户端演示了如何与部署在 Solana 区块链上的 Counter 智能合约进行交互
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction,
    signature::{Keypair, Signer},
    pubkey::Pubkey,
};
use borsh::{BorshDeserialize, BorshSerialize};
use borsh;

// 引用本地配置模块，用于加载程序配置（密钥、程序ID等）
mod config;
use config::initialize_program_config;

// 引用工具函数模块
mod utils;
use utils::{check_and_print_balance, send_transaction_and_check_balance, print_total_consumption};

/// Counter 账户的数据结构
/// 这个结构必须与智能合约中定义的 CounterAccount 结构完全一致
/// 使用 Borsh 进行序列化/反序列化，这是 Solana 推荐的序列化格式
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,  // 计数器的值，使用 64 位无符号整数
}

/// 智能合约支持的指令类型枚举
/// 这个枚举必须与智能合约中定义的 CounterInstruction 枚举完全一致
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    /// 初始化 counter 账户，将计数器设置为 0
    Initialize,
    /// 将计数器值加 1
    Increment,
    /// 将计数器值重置为 0（需要账户所有者签名）
    Reset,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Solana Counter 智能合约客户端启动 ===");
    
    // ========================================
    // 第一步：初始化配置
    // ========================================
    
    // 初始化程序配置，这会读取以下配置文件：
    // - 用户密钥对文件（用于签名交易）
    // - 程序ID文件（智能合约的地址）
    // - RPC配置文件（连接到哪个 Solana 网络）
    let config = initialize_program_config()?;
    println!("✅ 配置初始化成功!");
    println!("  - 程序ID: {}", config.program_id);
    println!("  - RPC URL: {}", config.rpc_url);
    println!("  - 用户地址: {}", config.keypair.pubkey());

    // ========================================
    // 第二步：连接到 Solana 网络
    // ========================================
    
    println!("\n🌐 连接到Solana网络: {}", config.rpc_url);
    // 创建 RPC 客户端，使用 confirmed 提交级别确保交易被确认
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());

    // 检查用户账户余额，确保有足够的 SOL 支付交易费用
    println!("\n💰 检查账户余额...");
    let balance = check_and_print_balance(&client, &config.keypair.pubkey(), "当前账户余额")?;
    let sol_balance = balance as f64 / 1_000_000_000.0; // 将 lamports 转换为 SOL（1 SOL = 10^9 lamports）
    
    // 检查余额是否足够（至少需要 0.01 SOL）
    if sol_balance < 0.01 {
        println!("⚠️  警告：账户余额可能不足以支付交易费用，建议至少有 0.01 SOL");
    }

    // ========================================
    // 第三步：获取用户的 Counter PDA 账户地址
    // ========================================
    
    // 计算用户的 Counter PDA 地址
    let (counter_pda, _bump_seed) = Pubkey::find_program_address(
        &[b"counter", config.keypair.pubkey().as_ref()],
        &config.program_id,
    );
    println!("\n📝 用户 Counter PDA 地址: {}", counter_pda);

    // 检查 Counter 账户是否已存在
    let counter_exists = match client.get_account(&counter_pda) {
        Ok(account) => {
            if account.lamports > 0 {
                let counter_data = CounterAccount::try_from_slice(&account.data)?;
                println!("✅ Counter 账户已存在，当前值: {}", counter_data.count);
                true
            } else {
                false
            }
        },
        Err(_) => {
            println!("ℹ️  Counter 账户不存在，需要创建");
            false
        }
    };

    // ========================================
    // 第四步：初始化 Counter（如果不存在）
    // ========================================
    
    if !counter_exists {
        println!("\n=== 步骤 1: 创建并初始化 Counter 账户 ===");
        
        // 序列化初始化指令数据
        let init_instruction_data = borsh::to_vec(&CounterInstruction::Initialize)?;
        
        // 创建初始化指令
        let init_instruction = Instruction::new_with_bytes(
            config.program_id,           
            &init_instruction_data,      
            vec![
                // Counter PDA 账户（可写，将被创建）
                AccountMeta::new(counter_pda, false),
                // 用户账户（可写，签名者，支付租金）
                AccountMeta::new(config.keypair.pubkey(), true),
                // 系统程序
                AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
            ],
        );

        // 构建并发送初始化交易
        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&config.keypair.pubkey()));
        transaction.sign(&[&config.keypair], recent_blockhash);

        let _signature = send_transaction_and_check_balance(
            &client,
            &transaction,
            &config.keypair.pubkey(),
            "Counter 创建和初始化"
        )?;

        // 读取并显示初始化后的 counter 值
        let counter_account = client.get_account(&counter_pda)?;
        let counter_data = CounterAccount::try_from_slice(&counter_account.data)?;
        println!("📊 初始化后 Counter 值: {}", counter_data.count);
    }

    // ========================================
    // 第五步：增加 Counter 三次
    // ========================================
    
    println!("\n=== 步骤 2: 增加 Counter（执行3次） ===");
    
    for i in 1..=3 {
        println!("\n🔄 第 {} 次增加操作:", i);
        
        // 序列化增加指令数据
        let increment_instruction_data = borsh::to_vec(&CounterInstruction::Increment)?;
        
        // 创建增加指令
        let increment_instruction = Instruction::new_with_bytes(
            config.program_id,
            &increment_instruction_data,
            vec![AccountMeta::new(counter_pda, false)], // 使用 PDA 地址
        );

        // 构建并发送增加交易
        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction = Transaction::new_with_payer(&[increment_instruction], Some(&config.keypair.pubkey()));
        transaction.sign(&[&config.keypair], recent_blockhash);

        let _signature = send_transaction_and_check_balance(
            &client,
            &transaction,
            &config.keypair.pubkey(),
            &format!("Counter 第{}次增加", i)
        )?;

        // 读取并显示更新后的 counter 值
        let counter_account = client.get_account(&counter_pda)?;
        let counter_data = CounterAccount::try_from_slice(&counter_account.data)?;
        println!("📊 当前 Counter 值: {}", counter_data.count);
    }
    // ========================================
    // 第六步：显示最终结果
    // ========================================
    
    // 读取并显示最终的 counter 值
    let counter_account = client.get_account(&counter_pda)?;
    let counter_data = CounterAccount::try_from_slice(&counter_account.data)?;
    println!("📊 最终 Counter 值: {}", counter_data.count);
    
    // 显示最终余额和总消耗
    let final_balance = check_and_print_balance(&client, &config.keypair.pubkey(), "最终余额")?;
    print_total_consumption(balance, final_balance);
    
    // ========================================
    // 演示完成
    // ========================================
    
    println!("\n🎉 === Counter 智能合约演示完成 ===");
    println!("📝 本次演示执行的操作:");
    if !counter_exists {
        println!("   1. ✅ 创建了用户的 Counter PDA 账户");
    } else {
        println!("   1. ✅ 使用现有的 Counter PDA 账户");
    }
    println!("   2. ✅ 执行了 3 次增加操作");
    println!("🎊 所有操作均成功完成！");
    println!("ℹ️  您的 Counter PDA 地址: {}", counter_pda);
    println!("ℹ️  使用 close.rs 可以关闭账户并回收租金");
    
    Ok(())
}