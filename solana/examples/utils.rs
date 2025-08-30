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

// 引入src中的工具函数，避免重复实现
// 注意：这里需要使用相对路径引用同一crate中的模块
use gong_de_increase::utils::{
    GONGDE_VALUE_SIZE,
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

/// 从账户数据中读取功德值（客户端版本）
/// 这是对src版本的包装，提供客户端友好的错误处理
/// 
/// # 参数
/// * `account_data` - 账户数据字节数组
/// 
/// # 返回
/// * `u32` - 功德值，如果数据不足则返回0
pub fn read_gongde_value(account_data: &[u8]) -> u32 {
    // 使用src中的函数，但提供客户端友好的错误处理
    match gong_de_increase::utils::read_gongde_value(account_data) {
        Ok(value) => value,
        Err(_) => 0, // 客户端版本：数据不足时返回0而不是错误
    }
}

/// 生成用户的功德账户地址（客户端版本）
/// 这是对src版本的包装，提供客户端友好的错误处理
/// 
/// # 参数
/// * `user_pubkey` - 用户公钥
/// * `program_id` - 程序ID
/// 
/// # 返回
/// * `Result<Pubkey, Box<dyn std::error::Error>>` - 功德账户地址
pub fn get_gongde_account_address(
    user_pubkey: &Pubkey, 
    program_id: &Pubkey
) -> Result<Pubkey, Box<dyn std::error::Error>> {
    // 使用src中的函数，转换错误类型
    gong_de_increase::utils::derive_gongde_account_address(user_pubkey, program_id)
        .map_err(|e| format!("生成账户地址失败: {:?}", e).into())
}

/// 查询用户的功德账户信息
/// 
/// # 参数
/// * `client` - RPC客户端
/// * `user_pubkey` - 用户公钥
/// * `program_id` - 程序ID
/// 
/// # 返回
/// * `Result<Option<(Pubkey, u32, u64)>, Box<dyn std::error::Error>>` - 
///   返回 Some((账户地址, 功德值, 账户余额)) 如果账户存在，否则返回 None
pub fn query_gongde_account(
    client: &RpcClient,
    user_pubkey: &Pubkey,
    program_id: &Pubkey,
) -> Result<Option<(Pubkey, u32, u64)>, Box<dyn std::error::Error>> {
    // 生成功德账户地址
    let gongde_pubkey = get_gongde_account_address(user_pubkey, program_id)?;
    
    // 查询账户信息
    match client.get_account(&gongde_pubkey) {
        Ok(account) => {
            if account.lamports > 0 && account.data.len() >= GONGDE_VALUE_SIZE {
                let gongde_value = read_gongde_value(&account.data);
                Ok(Some((gongde_pubkey, gongde_value, account.lamports)))
            } else {
                Ok(None)
            }
        },
        Err(_) => Ok(None)
    }
}

/// 格式化并打印功德账户信息
/// 
/// # 参数
/// * `user_pubkey` - 用户公钥
/// * `gongde_info` - 功德账户信息 (账户地址, 功德值, 账户余额)
pub fn print_gongde_info(user_pubkey: &Pubkey, gongde_info: Option<(Pubkey, u32, u64)>) {
    println!("👤 用户地址: {}", user_pubkey);
    
    match gongde_info {
        Some((gongde_pubkey, gongde_value, account_balance)) => {
            println!("✅ 功德账户已存在");
            println!("📍 功德账户地址: {}", gongde_pubkey);
            println!("🙏 当前功德值: {}", gongde_value);
            println!("💰 账户余额: {}", format_sol_balance(account_balance));
            
            // 功德等级判断
            let level = match gongde_value {
                0 => "🥉 初心",
                1..=10 => "🥈 善念",
                11..=100 => "🥇 善行",
                101..=1000 => "🏆 德高",
                1001..=10000 => "💎 圣贤",
                _ => "🌟 功德圆满"
            };
            println!("🏅 功德等级: {}", level);
        },
        None => {
            println!("❌ 功德账户不存在");
            println!("💡 提示: 可以使用 client.rs 创建功德账户");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
