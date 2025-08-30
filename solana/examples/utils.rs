// ========================================
// 工具函数模块
// 包含客户端常用的工具函数，如余额检查、交易发送等
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction,
    signature::Signature,
};

/// 检查并打印账户余额
/// 
/// # 参数
/// * `client` - RPC客户端
/// * `pubkey` - 要检查的账户公钥
/// * `label` - 余额标签（用于日志输出）
/// 
/// # 返回
/// * `Result<u64, Box<dyn std::error::Error>>` - 账户余额（lamports）
pub fn check_and_print_balance(
    client: &RpcClient,
    pubkey: &Pubkey,
    label: &str,
) -> Result<u64, Box<dyn std::error::Error>> {
    let balance = client.get_balance(pubkey)?;
    let sol_balance = balance as f64 / 1_000_000_000.0;
    println!("💰 {}: {:.6} SOL ({} lamports)", label, sol_balance, balance);
    Ok(balance)
}

/// 发送交易并检查余额变化
/// 
/// # 参数
/// * `client` - RPC客户端
/// * `transaction` - 要发送的交易
/// * `payer_pubkey` - 交易费用付费者的公钥
/// * `operation_name` - 操作名称（用于日志输出）
/// 
/// # 返回
/// * `Result<Signature, Box<dyn std::error::Error>>` - 交易签名
pub fn send_transaction_and_check_balance(
    client: &RpcClient,
    transaction: &Transaction,
    payer_pubkey: &Pubkey,
    operation_name: &str,
) -> Result<Signature, Box<dyn std::error::Error>> {
    // 发送交易并等待确认
    let signature = client.send_and_confirm_transaction(transaction)?;
    println!("✅ {} 成功!", operation_name);
    println!("📝 {} 交易签名: {}", operation_name, signature);
    
    // 检查操作后的余额
    check_and_print_balance(client, payer_pubkey, &format!("{}后余额", operation_name))?;
    
    Ok(signature)
}

/// 计算并打印总消耗
/// 
/// # 参数
/// * `initial_balance` - 初始余额（lamports）
/// * `final_balance` - 最终余额（lamports）
pub fn print_total_consumption(initial_balance: u64, final_balance: u64) {
    let total_consumed = initial_balance.saturating_sub(final_balance);
    let total_consumed_sol = total_consumed as f64 / 1_000_000_000.0;
    println!("📉 总消耗: {:.6} SOL ({} lamports)", total_consumed_sol, total_consumed);
}

/// 将lamports转换为SOL并格式化显示
/// 
/// # 参数
/// * `lamports` - lamports数量
/// 
/// # 返回
/// * `String` - 格式化的SOL字符串
pub fn format_sol_balance(lamports: u64) -> String {
    let sol_balance = lamports as f64 / 1_000_000_000.0;
    format!("{:.6} SOL", sol_balance)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
