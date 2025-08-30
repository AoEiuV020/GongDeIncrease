// ========================================
// Solana Counter 智能合约客户端（最简版）
// 🎯 这个文件演示如何调用智能合约，类比传统的函数调用：
// 传统调用：counter.increment()
// Solana调用：发送Instruction到网络，包含"函数名"和"参数"
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},  // 🎯 这是调用合约的"指令"结构
    transaction::Transaction,                 // 📦 交易包装器
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

// 🎯 定义"函数名"常量 - 类比函数名枚举
// 这些数字对应合约中的指令类型
const INSTRUCTION_INCREMENT: u8 = 0;  // 对应合约中的increment函数

// 📖 从账户数据中读取counter值的辅助函数
// 类比：从对象中读取属性值
fn read_counter_value(account_data: &[u8]) -> u32 {
    if account_data.len() >= 4 {
        // 将字节数组转换为u32数字（小端序）
        u32::from_le_bytes([
            account_data[0], account_data[1], account_data[2], account_data[3]
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

    // 📱 连接到 Solana 网络 - 类比连接数据库
    println!("\n🌐 连接到Solana网络: {}", config.rpc_url);
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());

    // 💰 检查账户余额
    println!("\n💰 检查账户余额...");
    let balance = check_and_print_balance(&client, &config.keypair.pubkey(), "当前账户余额")?;

    // 🏗️ 创建专属的数据账户地址
    // 类比：为每个用户创建专属的数据存储空间
    // 使用 create_account_with_seed 方案，更简单且不需要合约支持
    let seed = "counter";
    let counter_pubkey = Pubkey::create_with_seed(
        &config.keypair.pubkey(),  // 基础地址（用户公钥）
        seed,                      // 种子字符串
        &config.program_id,        // 合约程序ID
    )?;
    println!("\n📝 用户专属 Counter 账户地址: {}", counter_pubkey);
    println!("   (基于用户公钥 + 种子: '{}' + 程序ID生成)", seed);

    // 🔍 检查 Counter 账户是否已存在
    // 类比：检查对象是否已经被创建
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

    // 🏗️ 如果账户不存在，创建新的数据账户
    // 类比：如果对象不存在，先new一个对象
    if !counter_exists {
        println!("\n=== 步骤 1: 创建 Counter 账户 ===");
        
        // 💰 计算账户所需租金（4字节数据空间）
        // Solana上存储数据需要支付租金，防止垃圾数据
        let rent = client.get_minimum_balance_for_rent_exemption(4)?;
        
        // 🏗️ 使用系统程序创建账户（不是调用我们的合约）
        let create_instruction = system_instruction::create_account_with_seed(
            &config.keypair.pubkey(), // 付款账户
            &counter_pubkey,          // 新账户地址
            &config.keypair.pubkey(), // 基础账户
            seed,                     // 种子字符串
            rent,                     // 租金金额
            4,                        // 数据空间大小（4字节存u32）
            &config.program_id,       // 账户所有者（我们的合约程序）
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
            "Counter 账户创建"
        )?;

        println!("✅ Counter 账户创建成功，初始值: 0");
        counter_exists = true;
    }

    // 只有在账户存在时才继续执行
    if counter_exists {

    // 🎯 调用智能合约函数 - 增加 Counter 三次
    // 类比：多次调用 counter.increment() 方法
    println!("\n=== 步骤 2: 增加 Counter（执行3次） ===");
    
    for i in 1..=3 {
        println!("\n🔄 第 {} 次增加操作:", i);
        
        // 📝 创建调用指令 - 这就是"函数调用"的核心
        // 类比：准备函数调用 counter.increment()
        let increment_instruction = Instruction::new_with_bytes(
            config.program_id,                              // 🎯 合约地址（类似类名）
            &[INSTRUCTION_INCREMENT],                       // 📋 "函数名"：0表示increment函数
            vec![AccountMeta::new(counter_pubkey, false)],  // 📁 "参数"：需要操作的账户
        );
        // 📝 AccountMeta::new(地址, 是否需要签名) 表示一个可写的账户参数

        // 📤 发送交易到网络执行
        // 类比：实际执行函数调用
        let recent_blockhash = client.get_latest_blockhash()?;
        let mut transaction = Transaction::new_with_payer(
            &[increment_instruction],  // 包含我们的指令
            Some(&config.keypair.pubkey())  // 交易付费者
        );
        transaction.sign(&[&config.keypair], recent_blockhash);  // 数字签名

        let _signature = send_transaction_and_check_balance(
            &client,
            &transaction,
            &config.keypair.pubkey(),
            &format!("Counter 第{}次增加", i)
        )?;

        // 📊 读取函数执行结果 - 查看counter的新值
        // 类比：获取函数执行后对象的状态
        let counter_account = client.get_account(&counter_pubkey)?;
        let counter_value = read_counter_value(&counter_account.data);
        println!("📊 当前 Counter 值: {}", counter_value);
    }

    // 📊 显示最终结果和统计信息
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
    
    } // 结束if counter_exists的代码块
    
    Ok(())
}

// ========================================
// 💡 Solana智能合约调用总结（类比传统函数调用）
// ========================================
// 
// 🎯 传统函数调用 vs Solana合约调用：
// 
// 传统方式：
//   counter.increment()  // 直接调用对象的方法
// 
// Solana方式：
//   1. 创建Instruction（指令）
//      - program_id: 合约地址（类似类名）
//      - data: [0] 表示调用increment函数（函数名编码）
//      - accounts: [counter_account] 需要操作的账户（函数参数）
//   
//   2. 包装成Transaction（交易）
//   3. 签名并发送到网络
//   4. 网络执行合约中的process_instruction函数
//   5. 根据data[0]的值，路由到对应的处理逻辑
// 
// 🔑 关键概念对照：
//   - program_id ≈ 类名/合约地址
//   - instruction_data ≈ 函数名+参数的序列化
//   - accounts ≈ 函数需要访问的对象引用
//   - Transaction ≈ 原子操作包装器
//   - 网络执行 ≈ 函数调用的实际执行
// 
// 📝 这种设计的优势：
//   - 所有状态变更都可追溯（区块链特性）
//   - 并发执行优化（账户模型）
//   - 确定性执行（所有操作可重现）
// ========================================
