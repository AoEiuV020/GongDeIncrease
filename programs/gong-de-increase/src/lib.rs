use anchor_lang::prelude::*;

declare_id!("9jpqDtrTj4GyNLVDjydbJVW1pWkZypHwpqDyLt2Ragt9");

#[program]
pub mod gong_de_increase {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
