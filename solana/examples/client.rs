use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    transaction::Transaction,
    signature::Signer,
};

// 引用本地配置模块
mod config;
use config::initialize_program_config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Solana 程序客户端启动 ===");
    
    // 初始化程序配置（读取所有配置文件）
    let config = initialize_program_config()?;
    println!("配置初始化成功!");
    println!("  - 程序ID: {}", config.program_id);
    println!("  - RPC URL: {}", config.rpc_url);
    println!("  - 用户地址: {}", config.keypair.pubkey());

    // 连接到Solana网络
    println!("\n连接到Solana网络: {}", config.rpc_url);
    let client = RpcClient::new_with_commitment(config.rpc_url, CommitmentConfig::confirmed());

    // 检查用户账户余额
    println!("\n检查账户余额...");
    let balance = client.get_balance(&config.keypair.pubkey())?;
    let sol_balance = balance as f64 / 1_000_000_000.0; // 转换为SOL
    println!("当前账户余额: {} SOL ({} lamports)", sol_balance, balance);

    // 创建调用程序的指令
    println!("\n创建程序调用指令...");
    let instruction = Instruction::new_with_borsh(
        config.program_id,
        &(), // 空的指令数据 - 根据您的程序需求修改
        vec![], // 无需额外账户 - 根据您的程序需求修改
    );

    // 构建并签名交易
    println!("构建交易...");
    let recent_blockhash = client.get_latest_blockhash()?;
    println!("获取最新区块哈希: {}", recent_blockhash);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&config.keypair.pubkey()));
    transaction.sign(&[&config.keypair], recent_blockhash);

    // 发送并确认交易
    println!("\n发送交易到区块链...");
    let signature = client.send_and_confirm_transaction(&transaction)?;
    println!("✅ 交易成功!");
    println!("交易签名: {}", signature);
    println!("可以在区块链浏览器中查看交易详情");
    
    println!("\n=== 程序执行完成 ===");
    Ok(())
}