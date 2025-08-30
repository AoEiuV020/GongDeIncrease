// ========================================
// Solana 功德 智能合约客户端（支持全局PDA账户）
// 🎯 这个文件演示如何调用智能合约，包含全局PDA账户的自动创建
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction,
    signature::Signer,
    system_instruction,
};

// 引用本地配置模块
mod config;
use config::initialize_program_config_with_keypair;

// 引用工具函数模块
mod utils;
use utils::{check_and_print_balance, send_transaction_and_check_balance, print_total_consumption};
use gong_de_increase::utils::{
    read_gongde_value, 
    derive_gongde_account_address, 
    derive_global_gongde_pda_address,
    GongDeInstruction, 
    GONGDE_VALUE_SIZE, 
    GONGDE_ACCOUNT_SEED, 
    GLOBAL_GONGDE_ACCOUNT_SEED,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Solana 功德 智能合约客户端（支持全局PDA）启动 ===");
    
    // 检查命令行参数
    let args: Vec<String> = std::env::args().collect();
    
    // 初始化配置 - 如果有第一个参数就使用指定的私钥文件，否则使用默认配置
    let config = if args.len() > 1 {
        let keypair_path = &args[1];
        println!("🔑 使用指定的私钥文件: {}", keypair_path);
        initialize_program_config_with_keypair(Some(keypair_path))?
    } else {
        println!("🔑 使用默认配置");
        initialize_program_config_with_keypair(None)?
    };
    println!("✅ 配置初始化成功!");
    println!("  - 程序ID: {}", config.program_id);
    println!("  - RPC URL: {}", config.rpc_url);
    println!("  - 用户地址: {}", config.keypair.pubkey());

    // 📱 连接到 Solana 网络
    println!("\n🌐 连接到Solana网络: {}", config.rpc_url);
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());

    // 💰 检查账户余额
    println!("\n💰 检查账户余额...");
    let balance = check_and_print_balance(&client, &config.keypair.pubkey(), "当前账户余额")?;

    // 🏗️ 创建专属的数据账户地址
    let gongde_pubkey = derive_gongde_account_address(&config.keypair.pubkey(), &config.program_id)
        .map_err(|e| format!("生成账户地址失败: {:?}", e))?;
    println!("\n📝 用户专属功德账户地址: {}", gongde_pubkey);
    println!("   (基于用户公钥 + 种子: '{}' + 程序ID生成)", GONGDE_ACCOUNT_SEED);

    // 🌍 创建全局功德PDA账户地址
    let (global_gongde_pubkey, _bump) = derive_global_gongde_pda_address(&config.program_id)
        .map_err(|e| format!("生成全局PDA账户地址失败: {:?}", e))?;
    println!("\n🌍 全局功德PDA账户地址: {}", global_gongde_pubkey);
    println!("   (基于程序ID + 种子: '{}' 的PDA)", GLOBAL_GONGDE_ACCOUNT_SEED);

    // 🔍 检查功德账户是否已存在
    let mut gongde_exists = match client.get_account(&gongde_pubkey) {
        Ok(account) => {
            if account.lamports > 0 {
                let gongde_value = read_gongde_value(&account.data)
                    .map_err(|e| format!("读取功德值失败: {:?}", e))?;
                println!("✅ 功德账户已存在，当前值: {}", gongde_value);
                true
            } else {
                false
            }
        },
        Err(_) => {
            println!("ℹ️  功德账户不存在，需要创建");
            false
        }
    };

    // 🔍 检查全局功德PDA账户是否已存在
    let global_gongde_exists = match client.get_account(&global_gongde_pubkey) {
        Ok(account) => {
            if account.lamports > 0 {
                let global_gongde_value = read_gongde_value(&account.data)
                    .map_err(|e| format!("读取全局功德值失败: {:?}", e))?;
                println!("✅ 全局功德PDA账户已存在，当前值: {}", global_gongde_value);
                true
            } else {
                false
            }
        },
        Err(_) => {
            println!("ℹ️  全局功德PDA账户不存在，将在第一次调用时自动创建");
            false
        }
    };

    // 🏗️ 如果个人账户不存在，创建新的数据账户
    if !gongde_exists {
        println!("\n=== 步骤 1: 创建个人功德账户 ===");
        
        // 💰 计算账户所需租金
        let rent = client.get_minimum_balance_for_rent_exemption(GONGDE_VALUE_SIZE)?;
        
        // 🏗️ 使用系统程序创建账户
        let create_instruction = system_instruction::create_account_with_seed(
            &config.keypair.pubkey(),
            &gongde_pubkey,
            &config.keypair.pubkey(),
            GONGDE_ACCOUNT_SEED,
            rent,
            GONGDE_VALUE_SIZE as u64,
            &config.program_id,
        );

        // 📦 构建并发送创建账户的交易
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
            "个人功德账户创建"
        )?;

        println!("✅ 个人功德账户创建成功，初始值: 0");
        gongde_exists = true;
    }

    // 只有在个人账户存在时才继续执行
    if gongde_exists {
        // 🎯 调用智能合约函数 - 增加功德三次
        println!("\n=== 步骤 2: 增加功德（执行3次） ===");
        if !global_gongde_exists {
            println!("注意：全局PDA账户将在第一次调用时自动创建");
        }
        
        for i in 1..=3 {
            println!("\n🔄 第 {} 次增加操作:", i);
            
            // 📝 创建调用指令 - 手续费将直接转到全局PDA账户
            let increment_instruction = Instruction::new_with_bytes(
                config.program_id,
                &[GongDeInstruction::Increment as u8],
                vec![
                    AccountMeta::new(gongde_pubkey, false),           // 个人功德账户（可写）
                    AccountMeta::new(config.keypair.pubkey(), true),  // 用户账户（签名者，支付手续费）
                    AccountMeta::new(global_gongde_pubkey, false),    // 全局PDA账户（可写）
                    AccountMeta::new_readonly(solana_sdk::system_program::id(), false), // 系统程序
                ],
            );

            // 📤 发送交易到网络执行
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
                &format!("功德第{}次增加", i)
            )?;

            // 📊 读取执行结果
            let gongde_account = client.get_account(&gongde_pubkey)?;
            let gongde_value = read_gongde_value(&gongde_account.data)
                .map_err(|e| format!("读取功德值失败: {:?}", e))?;
            println!("📊 当前个人功德值: {}", gongde_value);

            // 📊 读取全局功德值
            let global_gongde_account = client.get_account(&global_gongde_pubkey)?;
            let global_gongde_value = read_gongde_value(&global_gongde_account.data)
                .map_err(|e| format!("读取全局功德值失败: {:?}", e))?;
            println!("🌍 当前全局功德值: {}", global_gongde_value);
        }

        // 📊 显示最终结果和统计信息
        let gongde_account = client.get_account(&gongde_pubkey)?;
        let final_gongde_value = read_gongde_value(&gongde_account.data)
            .map_err(|e| format!("读取最终功德值失败: {:?}", e))?;
        println!("\n📊 最终个人功德值: {}", final_gongde_value);

        let global_gongde_account = client.get_account(&global_gongde_pubkey)?;
        let final_global_gongde_value = read_gongde_value(&global_gongde_account.data)
            .map_err(|e| format!("读取最终全局功德值失败: {:?}", e))?;
        println!("🌍 最终全局功德值: {}", final_global_gongde_value);

        // 显示最终余额和总消耗
        let final_balance = check_and_print_balance(&client, &config.keypair.pubkey(), "最终余额")?;
        print_total_consumption(balance, final_balance);
        
        println!("\n🎉 === 功德智能合约演示完成 ===");
        println!("📝 本次演示执行的操作:");
        println!("   1. ✅ 创建了用户的个人功德账户");
        println!("   2. ✅ 自动创建了全局功德PDA账户");
        println!("   3. ✅ 执行了 3 次增加操作（同时增加个人和全局功德）");
        println!("🎊 所有操作均成功完成！");
        println!("ℹ️  个人功德账户地址: {}", gongde_pubkey);
        println!("ℹ️  全局功德PDA账户地址: {}", global_gongde_pubkey);
        println!("ℹ️  使用 close.rs 可以关闭个人账户并回收租金");
    }
    
    Ok(())
}
