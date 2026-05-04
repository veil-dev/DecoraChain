#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, Address, DecoraChainClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DecoraChain);
        let client = DecoraChainClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        (env, user, client)
    }

    // ── Test 1: Happy Path ────────────────────────────────────
    // Full MVP: submit → 10 upvotes → claim
    #[test]
    fn test_submit_upvote_and_claim() {
        let (env, designer, client) = setup();
        let voter = Address::generate(&env);

        let count = client.submit_design(
            &designer,
            &String::from_str(&env, "QmXyZ123abc"),
            &symbol_short!("boho"),
        );
        assert_eq!(count, 1);

        for _ in 0..10 {
            client.upvote_design(&voter, &designer);
        }

        assert_eq!(client.get_reward_balance(&designer), 10);

        let claimed = client.claim_rewards(&designer);
        assert_eq!(claimed, 10);
        assert_eq!(client.get_reward_balance(&designer), 0);
    }

    // ── Test 2: Edge Case — upvote non-existent design ───────
    // Must emit contract error code 1, NOT a Wasm unreachable trap
    #[test]
    fn test_upvote_nonexistent_design_gives_contract_error() {
        let (env, voter, client) = setup();
        let ghost = Address::generate(&env);

        let result = client.try_upvote_design(&voter, &ghost);
        assert!(result.is_err());

        let sdk_err = result.unwrap_err().unwrap();
        assert_eq!(
            sdk_err,
            soroban_sdk::Error::from_contract_error(1), // DesignNotFound
            "Must be contract error 1, not a Wasm trap"
        );
    }

    // ── Test 3: State Verification ────────────────────────────
    // Stored design fields must exactly match submitted values
    #[test]
    fn test_design_state_after_submit() {
        let (env, designer, client) = setup();

        let hash  = String::from_str(&env, "QmSTATE789");
        let theme = symbol_short!("indstrl");

        client.submit_design(&designer, &hash, &theme);

        let d = client.get_design(&designer);
        assert_eq!(d.owner, designer);
        assert_eq!(d.layout_hash, hash);
        assert_eq!(d.upvotes, 0);
        assert!(!d.reward_claimed);
    }

    // ── Test 4: Claim Below Threshold ────────────────────────
    // claim_rewards with < 10 DCOR must return contract error 2
    #[test]
    fn test_claim_below_threshold_gives_contract_error() {
        let (env, designer, client) = setup();
        let voter = Address::generate(&env);

        client.submit_design(
            &designer,
            &String::from_str(&env, "QmFEW"),
            &symbol_short!("minimal"),
        );

        for _ in 0..5 {
            client.upvote_design(&voter, &designer);
        }

        let result = client.try_claim_rewards(&designer);
        assert!(result.is_err());

        let sdk_err = result.unwrap_err().unwrap();
        assert_eq!(
            sdk_err,
            soroban_sdk::Error::from_contract_error(2), // BelowClaimMin
        );
    }

    // ── Test 5: has_design guard + global counter ─────────────
    // has_design returns false before submit, true after.
    // Global counter increments correctly across two users.
    #[test]
    fn test_has_design_and_global_counter() {
        let (env, user1, client) = setup();
        let user2 = Address::generate(&env);

        assert!(!client.has_design(&user1));
        assert_eq!(client.get_design_count(), 0);

        client.submit_design(
            &user1,
            &String::from_str(&env, "QmUSER1"),
            &symbol_short!("boho"),
        );
        assert!(client.has_design(&user1));
        assert_eq!(client.get_design_count(), 1);

        client.submit_design(
            &user2,
            &String::from_str(&env, "QmUSER2"),
            &symbol_short!("minimal"),
        );
        assert_eq!(client.get_design_count(), 2);
    }
}