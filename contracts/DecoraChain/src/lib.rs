#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    panic_with_error, contracterror,
    Address, Env, String, Symbol,
};

// ─────────────────────────────────────────────
//  Soroban-native error codes
//  #[contracterror] is the ONLY way to define
//  errors that work with soroban_sdk::Error
// ─────────────────────────────────────────────
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    DesignNotFound = 1,
    BelowClaimMin  = 2,
    AlreadyClaimed = 3,
}

// ─────────────────────────────────────────────
//  Storage keys
// ─────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Design(Address),
    ThemeVotes(Symbol),
    RewardBalance(Address),
    DesignCount,
}

// ─────────────────────────────────────────────
//  Data struct
// ─────────────────────────────────────────────
#[contracttype]
#[derive(Clone)]
pub struct RoomDesign {
    pub owner: Address,
    pub layout_hash: String,
    pub theme: Symbol,
    pub upvotes: u32,
    pub reward_claimed: bool,
}

// ─────────────────────────────────────────────
//  Contract — all fns return plain types.
//  Failures use panic_with_error! which emits
//  a proper contract error (not a Wasm trap).
// ─────────────────────────────────────────────
#[contract]
pub struct DecoraChain;

#[contractimpl]
impl DecoraChain {

    /// Submit a room design. Returns new global design count.
    pub fn submit_design(
        env: Env,
        owner: Address,
        layout_hash: String,
        theme: Symbol,
    ) -> u32 {
        owner.require_auth();

        let design = RoomDesign {
            owner: owner.clone(),
            layout_hash,
            theme,
            upvotes: 0,
            reward_claimed: false,
        };

        env.storage().persistent().set(&DataKey::Design(owner.clone()), &design);

        if !env.storage().persistent().has(&DataKey::RewardBalance(owner.clone())) {
            env.storage().persistent().set(&DataKey::RewardBalance(owner.clone()), &0u64);
        }

        let count: u32 = env.storage().persistent()
            .get(&DataKey::DesignCount).unwrap_or(0);
        let new_count = count + 1;
        env.storage().persistent().set(&DataKey::DesignCount, &new_count);

        new_count
    }

    /// Upvote a design. Awards 1 DCOR to the designer.
    /// Panics with Error::DesignNotFound (code 1) — not a Wasm trap.
    pub fn upvote_design(env: Env, voter: Address, designer: Address) {
        voter.require_auth();

        let maybe_design: Option<RoomDesign> = env.storage().persistent()
            .get(&DataKey::Design(designer.clone()));

        let mut design = match maybe_design {
            Some(d) => d,
            None => panic_with_error!(&env, Error::DesignNotFound),
        };

        design.upvotes += 1;
        env.storage().persistent().set(&DataKey::Design(designer.clone()), &design);

        let balance: u64 = env.storage().persistent()
            .get(&DataKey::RewardBalance(designer.clone())).unwrap_or(0);
        env.storage().persistent()
            .set(&DataKey::RewardBalance(designer.clone()), &(balance + 1));
    }

    /// Claim accumulated DCOR. Requires >= 10.
    /// Panics with Error::BelowClaimMin (code 2) if threshold not met.
    pub fn claim_rewards(env: Env, owner: Address) -> u64 {
        owner.require_auth();

        let balance: u64 = env.storage().persistent()
            .get(&DataKey::RewardBalance(owner.clone())).unwrap_or(0);

        if balance < 10 {
            panic_with_error!(&env, Error::BelowClaimMin);
        }

        // Mark reward_claimed on the design if it exists
        let maybe_design: Option<RoomDesign> = env.storage().persistent()
            .get(&DataKey::Design(owner.clone()));
        if let Some(mut design) = maybe_design {
            design.reward_claimed = true;
            env.storage().persistent().set(&DataKey::Design(owner.clone()), &design);
        }

        env.storage().persistent().set(&DataKey::RewardBalance(owner.clone()), &0u64);

        balance
    }

    /// Vote for a community theme.
    pub fn vote_theme(env: Env, voter: Address, theme: Symbol) {
        voter.require_auth();

        let current: u32 = env.storage().persistent()
            .get(&DataKey::ThemeVotes(theme.clone())).unwrap_or(0);
        env.storage().persistent().set(&DataKey::ThemeVotes(theme), &(current + 1));
    }

    // ── READ-ONLY ─────────────────────────────────────────────

    /// Returns the design for owner.
    /// Panics with Error::DesignNotFound (code 1) — clean contract error, not Wasm trap.
    /// Call has_design() first if you want to avoid the error.
    pub fn get_design(env: Env, owner: Address) -> RoomDesign {
        let maybe: Option<RoomDesign> = env.storage().persistent()
            .get(&DataKey::Design(owner));
        match maybe {
            Some(d) => d,
            None => panic_with_error!(&env, Error::DesignNotFound),
        }
    }

    /// Safe boolean check — always call this before get_design.
    pub fn has_design(env: Env, owner: Address) -> bool {
        env.storage().persistent().has(&DataKey::Design(owner))
    }

    /// Returns DCOR balance; 0 if never submitted.
    pub fn get_reward_balance(env: Env, owner: Address) -> u64 {
        env.storage().persistent()
            .get(&DataKey::RewardBalance(owner)).unwrap_or(0)
    }

    /// Returns vote count for a theme; 0 if never voted.
    pub fn get_theme_votes(env: Env, theme: Symbol) -> u32 {
        env.storage().persistent()
            .get(&DataKey::ThemeVotes(theme)).unwrap_or(0)
    }

    /// Returns total designs submitted globally.
    pub fn get_design_count(env: Env) -> u32 {
        env.storage().persistent()
            .get(&DataKey::DesignCount).unwrap_or(0)
    }
}