// ========================================
// Solana Counter 智能合约客户端（最简版）
// 直接使用用户钱包地址作为基础创建唯一账户
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction,
    signature::Signer,
    pubkey::Pubkey,
    system_instruction,
};

// 引用本地配置模块
mod config;
use config::initialize_program_config;

// 引用工具函数模块
mod utils;
use utils::{check_and_print_balance, send_transaction_and_check_balance, print_total_consumption};

// 指令类型：0=增加
const INSTRUCTION_INCREMENT: u8 = 0;

fn read_counter_value(account_data: &[u8]) -> u64 {
    if account_data.len() >= 8 {
        u64::from_le_bytes([
            account_data[0], account_data[1], account_data[2], account_data[3],
            account_data[4], account_data[5], account_data[6], account_data[7]
        ])
    } else {
        0
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Solana Counter 智能合约客户端（最简版）启动 ===");
    
    // 初始化配置
    let config = initialize_program_config()?;
    println!("✅ 配置初始化成功!");
    println!("  - 程序ID: {}", config.program_id);
    println!("  - RPC URL: {}", config.rpc_url);
    println!("  - 用户地址: {}", config.keypair.pubkey());

    // 连接到 Solana 网络
    println!("\n🌐 连接到Solana网络: {}", config.rpc_url);
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());

    // 检查账户余额
    println!("\n💰 检查账户余额...");
    let balance = check_and_print_balance(&client, &config.keypair.pubkey(), "当前账户余额")?;

    // 使用 create_account_with_seed 方案，更简单且不需要合约支持
    let seed = "counter";
    let counter_pubkey = Pubkey::create_with_seed(
        &config.keypair.pubkey(),
        seed,
        &config.program_id,
    )?;
    println!("\n📝 用户专属 Counter 账户地址: {}", counter_pubkey);
    println!("   (基于用户公钥 + 种子: '{}')", seed);

    // 检查 Counter 账户是否已存在
    let mut counter_exists = match client.get_account(&counter_pubkey) {
        Ok(account) => {
            if account.lamports > 0 {
                let counter_value = read_counter_value(&account.data);
                println!("✅ Counter 账户已存在，当前值: {}", counter_value);
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

    // 如果账户不存在，使用 create_account_with_seed 创建
    if !counter_exists {
        println!("\n=== 步骤 1: 创建 Counter 账户 ===");
        
        // 计算账户所需租金（8字节数据空间）
        let rent = client.get_minimum_balance_for_rent_exemption(8)?;
        
        // 使用 create_account_with_seed 创建账户
        let create_instruction = system_instruction::create_account_with_seed(
            &config.keypair.pubkey(), // funding account
            &counter_pubkey,          // new account
            &config.keypair.pubkey(), // base account
            seed,                     // seed
            rent,                     // lamports
            8,                        // space
            &config.program_id,       // owner
        );

        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction = Transaction::new_with_payer(
            &[create_instruction], 
            Some(&config.keypair.pubkey())
        );
        transaction.sign(&[&config.keypair], recent_blockhash);

        let _signature = send_transaction_and_check_balance(
            &client,
            &transaction,
            &config.keypair.pubkey(),
            "Counter 账户创建"
        )?;

        println!("✅ Counter 账户创建成功，初始值: 0");
        counter_exists = true;
    }

    // 增加 Counter 三次
    println!("\n=== 步骤 2: 增加 Counter（执行3次） ===");
    
    for i in 1..=3 {
        println!("\n🔄 第 {} 次增加操作:", i);
        
        // 创建增加指令
        let increment_instruction = Instruction::new_with_bytes(
            config.program_id,
            &[INSTRUCTION_INCREMENT],
            vec![AccountMeta::new(counter_pubkey, false)],
        );

        // 发送增加交易
        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction = Transaction::new_with_payer(
            &[increment_instruction], 
            Some(&config.keypair.pubkey())
        );
        transaction.sign(&[&config.keypair], recent_blockhash);

        let _signature = send_transaction_and_check_balance(
            &client,
            &transaction,
            &config.keypair.pubkey(),
            &format!("Counter 第{}次增加", i)
        )?;

        // 读取并显示更新后的值
        let counter_account = client.get_account(&counter_pubkey)?;
        let counter_value = read_counter_value(&counter_account.data);
        println!("📊 当前 Counter 值: {}", counter_value);
    }

    // 显示最终结果
    let counter_account = client.get_account(&counter_pubkey)?;
    let final_counter_value = read_counter_value(&counter_account.data);
    println!("\n📊 最终 Counter 值: {}", final_counter_value);
    
    // 显示最终余额和总消耗
    let final_balance = check_and_print_balance(&client, &config.keypair.pubkey(), "最终余额")?;
    print_total_consumption(balance, final_balance);
    
    println!("\n🎉 === Counter 智能合约演示完成 ===");
    println!("📝 本次演示执行的操作:");
    println!("   1. ✅ 创建了用户的 Counter 账户");
    println!("   2. ✅ 执行了 3 次增加操作");
    println!("🎊 所有操作均成功完成！");
    println!("ℹ️  Counter 账户地址: {}", counter_pubkey);
    println!("ℹ️  使用 close.rs 可以关闭账户并回收租金");
    
    Ok(())
}
