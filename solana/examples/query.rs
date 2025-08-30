// ========================================
// Solana 功德查询程序
// 🎯 根据用户公钥查询功德账户信息
// ========================================

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signer,
};
use std::env;

// 引用本地配置模块
mod config;
use config::initialize_program_config;

// 引用工具函数模块 - 直接使用src中的工具函数和examples中的客户端工具
mod utils;
use utils::{query_gongde_account, print_gongde_info};
use gong_de_increase::utils::GONGDE_VALUE_SIZE;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    println!("=== Solana 功德查询程序启动 ===");
    
    // 初始化配置（获取程序ID和RPC连接）
    let config = initialize_program_config()?;
    
    // 确定要查询的用户公钥
    let user_pubkey = if args.len() >= 2 {
        // 如果提供了公钥参数，解析并使用它
        let user_pubkey_str = &args[1];
        match user_pubkey_str.parse::<Pubkey>() {
            Ok(pubkey) => {
                println!("🔍 查询指定用户的功德: {}", pubkey);
                pubkey
            },
            Err(e) => {
                println!("❌ 错误: 无效的公钥格式: {}", e);
                println!("💡 公钥应该是58个字符的Base58编码字符串");
                println!("📖 用法: {} [用户公钥]", args[0]);
                println!("📝 示例: {} BvpjTs88TmXJrFfghPJmo1kEJXdtqXX8SdvW6jv8ng9R", args[0]);
                println!("💡 提示: 不提供公钥参数时将查询您自己的功德");
                return Err(e.into());
            }
        }
    } else {
        // 如果没有提供公钥参数，使用当前用户的公钥
        let user_pubkey = config.keypair.pubkey();
        println!("🔍 查询您自己的功德: {}", user_pubkey);
        user_pubkey
    };
    println!("✅ 配置初始化成功!");
    println!("  - 程序ID: {}", config.program_id);
    println!("  - RPC URL: {}", config.rpc_url);
    
    // 连接到 Solana 网络
    println!("\n🌐 连接到Solana网络: {}", config.rpc_url);
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());
    
    // 查询用户的功德账户
    println!("\n🔍 查询功德账户信息...");
    match query_gongde_account(&client, &user_pubkey, &config.program_id) {
        Ok(gongde_info) => {
            println!("\n📊 === 查询结果 ===");
            print_gongde_info(&user_pubkey, gongde_info);
            
            // 如果找到功德账户，显示详细统计
            if let Some((gongde_pubkey, gongde_value, account_balance)) = gongde_info {
                println!("\n📈 === 详细统计 ===");
                
                // 计算进度条
                let progress_chars = match gongde_value {
                    0 => "▱▱▱▱▱▱▱▱▱▱",
                    1..=10 => "▰▱▱▱▱▱▱▱▱▱",
                    11..=100 => "▰▰▱▱▱▱▱▱▱▱",
                    101..=1000 => "▰▰▰▱▱▱▱▱▱▱",
                    1001..=10000 => "▰▰▰▰▱▱▱▱▱▱",
                    _ => "▰▰▰▰▰▰▰▰▰▰"
                };
                
                println!("📊 功德进度: {} ({})", progress_chars, gongde_value);
                
                // 下一个等级所需功德
                let next_milestone = match gongde_value {
                    0..=10 => 11,
                    11..=100 => 101,
                    101..=1000 => 1001,
                    1001..=10000 => 10001,
                    _ => gongde_value
                };
                
                if gongde_value < u32::MAX && next_milestone > gongde_value {
                    let needed = next_milestone - gongde_value;
                    println!("🎯 距离下一等级还需: {} 功德", needed);
                }
                
                // 账户使用情况
                let rent_exemption = client.get_minimum_balance_for_rent_exemption(GONGDE_VALUE_SIZE)?;
                println!("💾 账户数据大小: {} 字节 (u32)", GONGDE_VALUE_SIZE);
                println!("💰 最低租金要求: {:.6} SOL", rent_exemption as f64 / 1_000_000_000.0);
                
                if account_balance >= rent_exemption {
                    println!("✅ 账户租金充足，数据安全");
                } else {
                    println!("⚠️  账户租金不足，数据可能被清理");
                }
                
                println!("\n🔗 区块链浏览器链接:");
                println!("   Solana Explorer: https://explorer.solana.com/address/{}?cluster=devnet", gongde_pubkey);
                println!("   Solscan: https://solscan.io/account/{}?cluster=devnet", gongde_pubkey);
            }
        },
        Err(e) => {
            println!("❌ 查询失败: {}", e);
            println!("💡 请检查网络连接或用户公钥是否正确");
            return Err(e);
        }
    }
    
    println!("\n🎉 === 查询完成 ===");
    println!("💡 提示:");
    println!("   - 使用 client.rs 可以创建和增加功德");
    println!("   - 使用 close.rs 可以关闭账户并回收租金");
    println!("   - 功德账户地址基于用户公钥确定性生成");
    
    Ok(())
}

// ========================================
// 💡 查询程序说明
// ========================================
// 
// 🎯 功能特点：
//   - 根据用户公钥查询功德账户
//   - 显示功德值和账户状态
//   - 提供功德等级和进度信息
//   - 显示区块链浏览器链接
// 
// 📋 使用方法：
//   cargo run --example query [用户公钥]
//   不提供公钥参数时查询自己的功德
// 
// 🔍 查询逻辑：
//   1. 解析用户公钥
//   2. 生成确定性的功德账户地址
//   3. 查询账户是否存在
//   4. 读取并解析功德数据
//   5. 格式化显示结果
// 
// 🏅 功德等级系统：
//   🥉 初心    : 0
//   🥈 善念    : 1-10
//   🥇 善行    : 11-100  
//   🏆 德高    : 101-1000
//   💎 圣贤    : 1001-10000
//   🌟 功德圆满 : 10000+
// 
// 🔐 安全特性：
//   - 只读查询，不消耗gas
//   - 不需要私钥，可查询任何用户
//   - 使用确定性地址生成
// ========================================
