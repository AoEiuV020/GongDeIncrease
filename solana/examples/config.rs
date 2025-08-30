use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signer}};
use std::fs;
use std::path::Path;
use serde::Deserialize;

/// Solana CLI 配置文件结构
#[derive(Debug, Deserialize)]
struct SolanaCliConfig {
    json_rpc_url: String,
    keypair_path: String,
    commitment: String,
}

/// 程序配置结构
#[derive(Debug)]
pub struct ProgramConfig {
    pub program_id: Pubkey,
    pub rpc_url: String,
    pub keypair: Keypair,
}

/// 从Solana CLI配置文件读取配置信息
/// 支持相对路径和绝对路径的私钥文件
fn load_solana_cli_config() -> Result<SolanaCliConfig, Box<dyn std::error::Error>> {
    // 尝试多个可能的配置文件路径
    let config_paths = [
        "./.config/solana/cli/config.yml",                    // 项目内配置
        "~/.config/solana/cli/config.yml",                   // 用户主目录配置
        &format!("{}/.config/solana/cli/config.yml", std::env::var("HOME").unwrap_or_default()),
    ];
    
    for path in &config_paths {
        let expanded_path = if path.starts_with("~/") {
            path.replace("~", &std::env::var("HOME").unwrap_or_default())
        } else {
            path.to_string()
        };
        
        if Path::new(&expanded_path).exists() {
            println!("读取Solana CLI配置文件: {}", expanded_path);
            let config_content = fs::read_to_string(&expanded_path)?;
            let config: SolanaCliConfig = serde_yaml::from_str(&config_content)?;
            return Ok(config);
        }
    }
    
    // 找不到配置文件直接报错
    Err("未找到Solana CLI配置文件，请检查以下路径:\n  - ./.config/solana/cli/config.yml\n  - ~/.config/solana/cli/config.yml".into())
}

/// 从私钥JSON文件加载密钥对
/// JSON文件格式为字节数组，例如: [1, 2, 3, ...]
fn load_keypair_from_file(file_path: &str) -> Result<Keypair, Box<dyn std::error::Error>> {
    // 处理相对路径
    let full_path = if file_path.starts_with("./") {
        file_path.to_string()
    } else if file_path.starts_with("~/") {
        file_path.replace("~", &std::env::var("HOME").unwrap_or_default())
    } else {
        file_path.to_string()
    };
    
    println!("从文件加载私钥: {}", full_path);
    
    if !Path::new(&full_path).exists() {
        return Err(format!("私钥文件不存在: {}", full_path).into());
    }
    
    let key_content = fs::read_to_string(&full_path)?;
    let key_bytes: Vec<u8> = serde_json::from_str(&key_content)?;
    
    if key_bytes.len() != 64 {
        return Err(format!("私钥文件格式错误，应包含64个字节，实际包含{}个字节", key_bytes.len()).into());
    }
    
    let keypair = Keypair::from_bytes(&key_bytes)?;
    println!("成功加载私钥，公钥地址: {}", keypair.pubkey());
    
    Ok(keypair)
}

/// 从部署文件中读取程序ID
/// 尝试多个可能的程序密钥对文件路径
fn load_program_id() -> Result<Pubkey, Box<dyn std::error::Error>> {
    // 可能的程序密钥对文件路径
    let program_keypair_paths = [
        "./target/deploy/gong_de_increase-keypair.json",
        "./solana/target/deploy/gong_de_increase-keypair.json",
        "../target/deploy/gong_de_increase-keypair.json",
    ];
    
    for path in &program_keypair_paths {
        if Path::new(path).exists() {
            println!("从部署文件读取程序ID: {}", path);
            let keypair_content = fs::read_to_string(path)?;
            let key_bytes: Vec<u8> = serde_json::from_str(&keypair_content)?;
            
            if key_bytes.len() != 64 {
                continue; // 尝试下一个文件
            }
            
            let program_keypair = Keypair::from_bytes(&key_bytes)?;
            let program_id = program_keypair.pubkey();
            println!("成功读取程序ID: {}", program_id);
            return Ok(program_id);
        }
    }
    
    // 找不到程序密钥对文件直接报错
    Err("未找到程序密钥对文件，请检查以下路径:\n  - ./target/deploy/gong_de_increase-keypair.json\n  - ./solana/target/deploy/gong_de_increase-keypair.json\n  - ../target/deploy/gong_de_increase-keypair.json".into())
}

/// 初始化程序配置
/// 从配置文件和密钥文件中读取所有必要的配置信息
pub fn initialize_program_config() -> Result<ProgramConfig, Box<dyn std::error::Error>> {
    // 1. 读取Solana CLI配置
    let cli_config = load_solana_cli_config()?;
    
    // 2. 加载程序ID
    let program_id = load_program_id()?;
    
    // 3. 加载用户私钥
    let keypair = load_keypair_from_file(&cli_config.keypair_path)?;
    
    Ok(ProgramConfig {
        program_id,
        rpc_url: cli_config.json_rpc_url,
        keypair,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
