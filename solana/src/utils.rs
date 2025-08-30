// ========================================
// 工具模块 - 共享的序列化反序列化和字节处理工具
// ========================================

use solana_program::{
    program_error::ProgramError,
    pubkey::Pubkey,
};

// ========================================
// 常量定义 - 消除硬编码
// ========================================

/// 功德值存储所需的字节数（u32类型需要4字节）
pub const GONGDE_VALUE_SIZE: usize = 4;

/// 功德账户种子字符串
pub const GONGDE_ACCOUNT_SEED: &str = "GongDeIncrease";

// ========================================
// 序列化反序列化工具函数
// ========================================

/// 从字节数组中读取功德值（u32，小端序）
/// 
/// # 参数
/// * `data` - 字节数据数组
/// 
/// # 返回
/// * `Result<u32, ProgramError>` - 解析的u32值
/// 
/// # 错误
/// * `ProgramError::AccountDataTooSmall` - 如果数据长度不足4字节
pub fn read_gongde_value(data: &[u8]) -> Result<u32, ProgramError> {
    if data.len() < GONGDE_VALUE_SIZE {
        return Err(ProgramError::AccountDataTooSmall);
    }
    
    Ok(u32::from_le_bytes([
        data[0], data[1], data[2], data[3]
    ]))
}

/// 将功德值写入字节数组（u32，小端序）
/// 
/// # 参数
/// * `data` - 目标字节数据数组（可变引用）
/// * `value` - 要写入的u32值
/// 
/// # 返回
/// * `Result<(), ProgramError>` - 成功返回Ok(())
/// 
/// # 错误
/// * `ProgramError::AccountDataTooSmall` - 如果数据长度不足4字节
pub fn write_gongde_value(data: &mut [u8], value: u32) -> Result<(), ProgramError> {
    if data.len() < GONGDE_VALUE_SIZE {
        return Err(ProgramError::AccountDataTooSmall);
    }
    
    let bytes = value.to_le_bytes();
    data[0..GONGDE_VALUE_SIZE].copy_from_slice(&bytes);
    Ok(())
}

/// 验证账户数据长度是否足够存储功德值
/// 
/// # 参数
/// * `data_len` - 账户数据长度
/// 
/// # 返回
/// * `Result<(), ProgramError>` - 验证通过返回Ok(())
/// 
/// # 错误
/// * `ProgramError::AccountDataTooSmall` - 如果数据长度不足
pub fn validate_account_data_size(data_len: usize) -> Result<(), ProgramError> {
    if data_len < GONGDE_VALUE_SIZE {
        return Err(ProgramError::AccountDataTooSmall);
    }
    Ok(())
}

// ========================================
// 账户地址生成工具函数
// ========================================

/// 生成用户的功德账户地址
/// 
/// # 参数
/// * `user_pubkey` - 用户公钥
/// * `program_id` - 程序ID
/// 
/// # 返回
/// * `Result<Pubkey, ProgramError>` - 功德账户地址
/// 
/// # 错误
/// * `ProgramError::InvalidSeeds` - 如果种子无效
pub fn derive_gongde_account_address(
    user_pubkey: &Pubkey, 
    program_id: &Pubkey
) -> Result<Pubkey, ProgramError> {
    Pubkey::create_with_seed(
        user_pubkey,           // 基础地址（用户公钥）
        GONGDE_ACCOUNT_SEED,   // 种子字符串
        program_id,            // 合约程序ID
    ).map_err(|_| ProgramError::InvalidSeeds)
}

// ========================================
// 指令类型枚举
// ========================================

/// 合约支持的指令类型
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GongDeInstruction {
    /// 增加功德值指令
    Increment = 0,
    /// 关闭账户指令
    Close = 1,
}

impl GongDeInstruction {
    /// 从字节解析指令类型
    /// 
    /// # 参数
    /// * `instruction_data` - 指令数据字节数组
    /// 
    /// # 返回
    /// * `Result<Self, ProgramError>` - 解析的指令类型
    /// 
    /// # 错误
    /// * `ProgramError::InvalidInstructionData` - 如果指令数据无效
    pub fn from_instruction_data(instruction_data: &[u8]) -> Result<Self, ProgramError> {
        match instruction_data.first().copied().unwrap_or(255) {
            0 => Ok(Self::Increment),
            1 => Ok(Self::Close),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_write_gongde_value() {
        let mut data = vec![0u8; 8]; // 8字节数据，够用
        let test_value = 12345u32;
        
        // 测试写入
        assert!(write_gongde_value(&mut data, test_value).is_ok());
        
        // 测试读取
        let read_value = read_gongde_value(&data).unwrap();
        assert_eq!(read_value, test_value);
    }
    
    #[test]
    fn test_insufficient_data_size() {
        let data = vec![0u8; 2]; // 只有2字节，不够
        
        // 测试读取失败
        assert_eq!(read_gongde_value(&data), Err(ProgramError::AccountDataTooSmall));
        
        // 测试数据大小验证失败
        assert_eq!(validate_account_data_size(2), Err(ProgramError::AccountDataTooSmall));
    }
    
    #[test]
    fn test_instruction_parsing() {
        // 测试有效指令
        assert_eq!(GongDeInstruction::from_instruction_data(&[0]), Ok(GongDeInstruction::Increment));
        assert_eq!(GongDeInstruction::from_instruction_data(&[1]), Ok(GongDeInstruction::Close));
        
        // 测试无效指令
        assert_eq!(GongDeInstruction::from_instruction_data(&[2]), Err(ProgramError::InvalidInstructionData));
        assert_eq!(GongDeInstruction::from_instruction_data(&[]), Err(ProgramError::InvalidInstructionData));
    }
}
