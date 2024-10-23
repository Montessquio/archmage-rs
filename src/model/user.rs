//!

pub struct User {
    guild_id: u64,
    user_id: u64,
    campaign: String,
    character: Option<String>,
    rank: u64,
    //render: UserRenderStyle
    xp_mul: f64,
}