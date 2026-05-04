#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, String};

    // ── Helper: register contract and return client ──────────
    fn setup() -> (Env, Address, DecoraChainClient<'static>) {
        let env = Env::default();
        env.mock_all_auths(); // auto-approve all require_auth() calls in tests
        let contract_id = env.register_contract(None, DecoraChain);
        let client = DecoraChainClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        (env, user, client)
    }

    // ── Test 1: Happy Path ───────────────────────────────────
    // A user submits a design, receives an upvote, and claims rewards.
    // This is the full MVP transaction flow end-to-end.
    #[test]
    fn test_submit_upvote_and_claim_rewards() {
        let (env, designer, client) = setup();
        let voter = Address::generate(&env);

        // Step 1: Submit a room design
        let count = client.submit_design(
            &designer,
            &String::from_str(&env, "QmXyZ123abc"),
            &symbol_short!("boho"),
        );
        assert_eq!(count, 1, "First design should set count to 1");

        // Step 2: Upvote the design 10 times (meets claim threshold)
        for _ in 0..10 {
            client.upvote_design(&voter, &designer);
        }

        // Step 3: Verify reward balance is 10
        let balance = client.get_reward_balance(&designer);
        assert_eq!(balance, 10, "Designer should have 10 DCOR after 10 upvotes");

        // Step 4: Claim the rewards
        let claimed = client.claim_rewards(&designer);
        assert_eq!(claimed, 10, "Should have claimed 10 DCOR");

        // Step 5: Balance should be reset to 0
        let post_balance = client.get_reward_balance(&designer);
        assert_eq!(post_balance, 0, "Balance should reset to 0 after claim");
    }

    // ── Test 2: Edge Case ────────────────────────────────────
    // Claiming rewards with fewer than 10 DCOR should panic.
    #[test]
    #[should_panic(expected = "Minimum 10 DCOR required to claim")]
    fn test_claim_below_threshold_fails() {
        let (env, designer, client) = setup();
        let voter = Address::generate(&env);

        // Submit design
        client.submit_design(
            &designer,
            &String::from_str(&env, "QmABC456"),
            &symbol_short!("minimal"),
        );

        // Only 5 upvotes — below the 10 DCOR threshold
        for _ in 0..5 {
            client.upvote_design(&voter, &designer);
        }

        // This should panic with threshold error
        client.claim_rewards(&designer);
    }

    // ── Test 3: State Verification ───────────────────────────
    // After submitting a design, all stored fields must match input exactly.
    #[test]
    fn test_design_state_after_submit() {
        let (env, designer, client) = setup();

        let hash = String::from_str(&env, "QmSTATE789");
        let theme = symbol_short!("indstrl");

        client.submit_design(&designer, &hash, &theme);

        let design = client.get_design(&designer);
        assert_eq!(design.owner, designer, "Owner address must match");
        assert_eq!(design.layout_hash, hash, "Layout hash must match");
        assert_eq!(design.upvotes, 0, "Upvotes should start at 0");
        assert_eq!(design.reward_claimed, false, "Reward claimed should start false");
    }

    // ── Test 4: Theme Voting ─────────────────────────────────
    // Multiple users voting on different themes should update tallies correctly.
    #[test]
    fn test_theme_vote_tallies() {
        let (env, user1, client) = setup();
        let user2 = Address::generate(&env);

        let boho = symbol_short!("boho");
        let minimal = symbol_short!("minimal");

        client.vote_theme(&user1, &boho);
        client.vote_theme(&user2, &boho);
        client.vote_theme(&user1, &minimal);

        assert_eq!(client.get_theme_votes(&boho), 2, "Boho should have 2 votes");
        assert_eq!(client.get_theme_votes(&minimal), 1, "Minimal should have 1 vote");
    }

    // ── Test 5: Global Design Counter ───────────────────────
    // Submitting multiple designs from different users increments global count.
    #[test]
    fn test_global_design_counter() {
        let (env, user1, client) = setup();
        let user2 = Address::generate(&env);

        assert_eq!(client.get_design_count(), 0, "Count starts at 0");

        client.submit_design(
            &user1,
            &String::from_str(&env, "QmUSER1"),
            &symbol_short!("boho"),
        );
        assert_eq!(client.get_design_count(), 1);

        client.submit_design(
            &user2,
            &String::from_str(&env, "QmUSER2"),
            &symbol_short!("minimal"),
        );
        assert_eq!(client.get_design_count(), 2, "Second submission increments to 2");
    }
}