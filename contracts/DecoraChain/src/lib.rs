#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
};

// ─────────────────────────────────────────────
//  Storage key types
// ─────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Design(Address),       // stores a user's submitted room design
    ThemeVotes(Symbol),    // tracks vote count for a given theme
    RewardBalance(Address),// XLM-equivalent reward points per user
    DesignCount,           // global counter of submitted designs
}

// ─────────────────────────────────────────────
//  Core data structures
// ─────────────────────────────────────────────

/// A room design submitted by a user
#[contracttype]
#[derive(Clone)]
pub struct RoomDesign {
    pub owner: Address,        // wallet address of the designer
    pub layout_hash: String,   // IPFS hash or unique hash of the room layout
    pub theme: Symbol,         // e.g. "minimalist", "boho", "industrial"
    pub upvotes: u32,          // community upvotes for this design
    pub reward_claimed: bool,  // whether the creator claimed their DCOR reward
}

// ─────────────────────────────────────────────
//  Contract
// ─────────────────────────────────────────────
#[contract]
pub struct DecoraChain;

#[contractimpl]
impl DecoraChain {

    /// Submit a new room design with a layout hash and theme.
    /// The designer must authorize this call.
    /// On success: design is stored, reward balance initialized,
    /// global design counter incremented.
    pub fn submit_design(
        env: Env,
        owner: Address,
        layout_hash: String,
        theme: Symbol,
    ) -> u32 {
        // Require the owner's signature — no one else can submit on their behalf
        owner.require_auth();

        // Build the design record
        let design = RoomDesign {
            owner: owner.clone(),
            layout_hash,
            theme: theme.clone(),
            upvotes: 0,
            reward_claimed: false,
        };

        // Persist the design keyed by owner address
        env.storage()
            .persistent()
            .set(&DataKey::Design(owner.clone()), &design);

        // Initialize reward balance to 0 DCOR tokens if not already set
        if !env
            .storage()
            .persistent()
            .has(&DataKey::RewardBalance(owner.clone()))
        {
            env.storage()
                .persistent()
                .set(&DataKey::RewardBalance(owner.clone()), &0u64);
        }

        // Increment and return the global design counter
        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::DesignCount)
            .unwrap_or(0);
        let new_count = count + 1;
        env.storage()
            .persistent()
            .set(&DataKey::DesignCount, &new_count);

        new_count
    }

    /// Upvote a room design owned by `designer`.
    /// The voter must authorize. Each upvote mints 1 DCOR point to the designer.
    pub fn upvote_design(env: Env, voter: Address, designer: Address) {
        // Voter must sign — prevents bot upvoting
        voter.require_auth();

        // Fetch the target design
        let mut design: RoomDesign = env
            .storage()
            .persistent()
            .get(&DataKey::Design(designer.clone()))
            .expect("Design not found");

        // Increment the upvote counter on the design
        design.upvotes += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Design(designer.clone()), &design);

        // Award 1 DCOR reward point to the designer for each upvote received
        let current_balance: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RewardBalance(designer.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::RewardBalance(designer.clone()), &(current_balance + 1));
    }

    /// Claim DCOR rewards accumulated from upvotes.
    /// Requires at least 10 DCOR to claim. Resets balance after claim.
    /// In production this triggers an XLM/USDC transfer via Stellar anchor.
    pub fn claim_rewards(env: Env, owner: Address) -> u64 {
        owner.require_auth();

        let balance: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::RewardBalance(owner.clone()))
            .unwrap_or(0);

        // Enforce minimum claim threshold of 10 DCOR points
        assert!(balance >= 10, "Minimum 10 DCOR required to claim");

        // Mark design's reward_claimed flag
        if env
            .storage()
            .persistent()
            .has(&DataKey::Design(owner.clone()))
        {
            let mut design: RoomDesign = env
                .storage()
                .persistent()
                .get(&DataKey::Design(owner.clone()))
                .unwrap();
            design.reward_claimed = true;
            env.storage()
                .persistent()
                .set(&DataKey::Design(owner.clone()), &design);
        }

        // Reset balance to 0 after claim
        env.storage()
            .persistent()
            .set(&DataKey::RewardBalance(owner.clone()), &0u64);

        balance // return the amount that was claimed
    }

    /// Cast a vote for a community theme (e.g. "boho", "industrial").
    /// Theme vote tallies guide the AI recommendation engine off-chain.
    pub fn vote_theme(env: Env, voter: Address, theme: Symbol) {
        voter.require_auth();

        let current: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::ThemeVotes(theme.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::ThemeVotes(theme), &(current + 1));
    }

    // ── Read-only views ──────────────────────────────────────

    /// Returns the room design for a given owner address
    pub fn get_design(env: Env, owner: Address) -> RoomDesign {
        env.storage()
            .persistent()
            .get(&DataKey::Design(owner))
            .expect("Design not found")
    }

    /// Returns the DCOR reward balance for a user
    pub fn get_reward_balance(env: Env, owner: Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::RewardBalance(owner))
            .unwrap_or(0)
    }

    /// Returns the vote count for a specific theme
    pub fn get_theme_votes(env: Env, theme: Symbol) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::ThemeVotes(theme))
            .unwrap_or(0)
    }

    /// Returns the total number of designs submitted globally
    pub fn get_design_count(env: Env) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::DesignCount)
            .unwrap_or(0)
    }
}