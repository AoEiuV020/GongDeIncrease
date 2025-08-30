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
    system_instruction,
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
    // 第三步：创建 Counter 数据账户
    // ========================================
    
    // 为 counter 数据创建一个新的账户密钥对
    // 这个账户将存储 counter 的状态数据
    let counter_keypair = Keypair::new();
    println!("\n📝 Counter 数据账户地址: {}", counter_keypair.pubkey());

    println!("\n=== 步骤 1: 创建 Counter 数据账户 ===");
    
    // 计算存储 CounterAccount 结构所需的空间
    let account_space = std::mem::size_of::<CounterAccount>();
    println!("所需存储空间: {} 字节", account_space);
    
    // 计算账户租金免除所需的最小余额
    // Solana 要求账户有足够的余额来免除租金，否则账户可能被删除
    let rent = client.get_minimum_balance_for_rent_exemption(account_space)?;
    println!("租金免除所需余额: {} lamports ({:.6} SOL)", rent, rent as f64 / 1_000_000_000.0);
    
    // 创建系统指令来创建新账户
    let create_account_instruction = system_instruction::create_account(
        &config.keypair.pubkey(),    // 付费者（将支付租金和交易费）
        &counter_keypair.pubkey(),   // 新账户的地址
        rent,                        // 转移到新账户的 lamports 数量
        account_space as u64,        // 账户数据空间大小
        &config.program_id,          // 账户所有者（我们的智能合约）
    );

    // 获取最新的区块哈希，这是交易的必需组件
    let recent_blockhash = client.get_latest_blockhash()?;
    
    // 构建交易
    let mut transaction = Transaction::new_with_payer(
        &[create_account_instruction],    // 交易中包含的指令
        Some(&config.keypair.pubkey()),   // 交易费用付费者
    );
    
    // 签名交易（需要付费者和新账户的签名）
    transaction.sign(&[&config.keypair, &counter_keypair], recent_blockhash);

    // 发送交易并检查余额
    let _signature = send_transaction_and_check_balance(
        &client, 
        &transaction, 
        &config.keypair.pubkey(), 
        "Counter 数据账户创建"
    )?;

    // ========================================
    // 第四步：初始化 Counter
    // ========================================
    
    println!("\n=== 步骤 2: 初始化 Counter ===");
    
    // 序列化初始化指令数据
    let init_instruction_data = borsh::to_vec(&CounterInstruction::Initialize)?;
    
    // 创建初始化指令
    let init_instruction = Instruction::new_with_bytes(
        config.program_id,           // 目标程序ID
        &init_instruction_data,      // 指令数据
        vec![
            // Counter 账户（可写，因为需要修改其数据）
            AccountMeta::new(counter_keypair.pubkey(), false),
            // 用户账户（只读，作为初始化的授权者）
            AccountMeta::new_readonly(config.keypair.pubkey(), true),
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
        "Counter 初始化"
    )?;

    // 读取并显示初始化后的 counter 值
    let counter_account = client.get_account(&counter_keypair.pubkey())?;
    let counter_data = CounterAccount::try_from_slice(&counter_account.data)?;
    println!("📊 初始化后 Counter 值: {}", counter_data.count);

    // ========================================
    // 第五步：多次增加 Counter
    // ========================================
    
    println!("\n=== 步骤 3: 增加 Counter（执行3次演示） ===");
    
    for i in 1..=3 {
        println!("\n🔄 第 {} 次增加操作:", i);
        
        // 序列化增加指令数据
        let increment_instruction_data = borsh::to_vec(&CounterInstruction::Increment)?;
        
        // 创建增加指令
        // 注意：增加操作只需要 Counter 账户，不需要用户签名
        let increment_instruction = Instruction::new_with_bytes(
            config.program_id,
            &increment_instruction_data,
            vec![AccountMeta::new(counter_keypair.pubkey(), false)], // 只需要 Counter 账户（可写）
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
        let counter_account = client.get_account(&counter_keypair.pubkey())?;
        let counter_data = CounterAccount::try_from_slice(&counter_account.data)?;
        println!("📊 当前 Counter 值: {}", counter_data.count);
    }

    // ========================================
    // 第六步：重置 Counter
    // ========================================
    
    println!("\n=== 步骤 4: 重置 Counter ===");
    
    // 序列化重置指令数据
    let reset_instruction_data = borsh::to_vec(&CounterInstruction::Reset)?;
    
    // 创建重置指令
    // 注意：重置操作需要用户签名作为授权
    let reset_instruction = Instruction::new_with_bytes(
        config.program_id,
        &reset_instruction_data,
        vec![
            // Counter 账户（可写）
            AccountMeta::new(counter_keypair.pubkey(), false),
            // 用户账户（只读，但需要签名作为重置授权）
            AccountMeta::new_readonly(config.keypair.pubkey(), true),
        ],
    );

    // 构建并发送重置交易
    let recent_blockhash = client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_with_payer(&[reset_instruction], Some(&config.keypair.pubkey()));
    transaction.sign(&[&config.keypair], recent_blockhash);

    let _signature = send_transaction_and_check_balance(
        &client,
        &transaction,
        &config.keypair.pubkey(),
        "Counter 重置"
    )?;

    // ========================================
    // 第七步：显示最终结果
    // ========================================
    
    // 读取并显示最终的 counter 值
    let counter_account = client.get_account(&counter_keypair.pubkey())?;
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
    println!("   1. ✅ 创建了 Counter 数据账户");
    println!("   2. ✅ 初始化 Counter 为 0");
    println!("   3. ✅ 执行了 3 次增加操作（0 → 1 → 2 → 3）");
    println!("   4. ✅ 重置 Counter 为 0");
    println!("🎊 所有操作均成功完成！");
    
    Ok(())
}