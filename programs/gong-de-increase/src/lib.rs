use anchor_lang::prelude::*;

declare_id!("GRnb9fxb2GeNnAG2ymTAhjtVZEXXUsV4NoBrbs3GxRUU");

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
