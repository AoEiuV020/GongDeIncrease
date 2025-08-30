// ========================================
// 关闭账户并回收租金（精简版）
// 用于关闭 Counter 账户并回收租金
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction,
    signature::Signer,
    pubkey::Pubkey,
};

// 引用本地配置模块
mod config;
use config::initialize_program_config;

// 引用工具函数模块
mod utils;
use utils::{check_and_print_balance, send_transaction_and_check_balance};

// 指令类型：1=关闭
const INSTRUCTION_CLOSE: u8 = 1;

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
    println!("=== 关闭 Counter 账户并回收租金（精简版）===");
    
    // 初始化配置
    let config = initialize_program_config()?;
    println!("✅ 配置初始化成功!");
    println!("  - 程序ID: {}", config.program_id);
    println!("  - 用户地址: {}", config.keypair.pubkey());

    // 连接到 Solana 网络
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());

    // 计算用户专属的 Counter PDA 地址
    let (counter_pubkey, _bump_seed) = Pubkey::find_program_address(
        &[b"counter", config.keypair.pubkey().as_ref()],
        &config.program_id,
    );
    println!("\n📝 用户专属 Counter PDA 地址: {}", counter_pubkey);

    // 检查 Counter 账户是否存在
    let _counter_account = match client.get_account(&counter_pubkey) {
        Ok(account) => {
            if account.lamports > 0 {
                let counter_value = read_counter_value(&account.data);
                println!("✅ Counter 账户存在，当前值: {}", counter_value);
                println!("📊 账户余额: {} lamports ({:.6} SOL)", 
                         account.lamports, 
                         account.lamports as f64 / 1_000_000_000.0);
                account
            } else {
                println!("❌ Counter 账户已经被关闭");
                return Ok(());
            }
        },
        Err(_) => {
            println!("❌ Counter 账户不存在，无需关闭");
            return Ok(());
        }
    };

    // 检查用户余额
    println!("\n💰 检查用户余额...");
    let balance_before = check_and_print_balance(&client, &config.keypair.pubkey(), "关闭前余额")?;

    // 关闭账户并回收租金
    println!("\n🔄 执行关闭操作...");
    
    // 创建关闭指令
    let close_instruction = Instruction::new_with_bytes(
        config.program_id,
        &[INSTRUCTION_CLOSE],
        vec![
            // Counter 账户（可写，将被关闭）
            AccountMeta::new(counter_pubkey, false),
            // 用户账户（可写，接收租金，签名者）
            AccountMeta::new(config.keypair.pubkey(), true),
        ],
    );

    // 发送关闭交易
    let recent_blockhash = client.get_latest_blockhash()?;
    let mut transaction = Transaction::new_with_payer(&[close_instruction], Some(&config.keypair.pubkey()));
    transaction.sign(&[&config.keypair], recent_blockhash);

    let _signature = send_transaction_and_check_balance(
        &client,
        &transaction,
        &config.keypair.pubkey(),
        "关闭 Counter 账户"
    )?;

    // 检查关闭后的余额
    let balance_after = check_and_print_balance(&client, &config.keypair.pubkey(), "关闭后余额")?;

    // 计算回收的租金
    let recovered_rent = balance_after.saturating_sub(balance_before);
    println!("🎉 成功回收租金: {} lamports ({:.6} SOL)", 
             recovered_rent, recovered_rent as f64 / 1_000_000_000.0);

    // 验证账户已被关闭
    match client.get_account(&counter_pubkey) {
        Ok(account) => {
            if account.lamports == 0 {
                println!("✅ 确认：账户已成功关闭");
            } else {
                println!("⚠️  警告：账户仍有余额");
            }
        },
        Err(_) => println!("✅ 确认：账户已完全删除"),
    }

    println!("\n🎊 账户关闭完成！租金已成功回收！");
    
    Ok(())
}