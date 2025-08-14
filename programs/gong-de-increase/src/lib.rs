use anchor_lang::prelude::*;

declare_id!("DbjZE2EpdkEbM42Q8VA5XB2yRpeSLZgYGyCi14DF6oYZ");

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
