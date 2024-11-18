use anchor_lang::prelude::*;

declare_id!("3kZmRLYfSmV9sYTH7upYwtLGmKkXRRMvoXwQLRnPchm2");

#[program]
pub mod raydium_puppet {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        todo!()
    }
}

#[derive(Accounts)]
pub struct Initialize {}
