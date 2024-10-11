#![cfg(test)]

use std::ops::Add;

use crate::{factor_instances_provider::provider::test_sargon_os::SargonOS, prelude::*};

type Sut = FactorInstancesProvider;

#[should_panic]
#[actix_rt::test]
async fn mfa_panics_if_entities_empty() {
    let fs = HDFactorSource::fs0();
    let a = Account::sample_unsecurified();
    let _ = Sut::for_account_mfa(
        &mut FactorInstancesCache::default(),
        MatrixOfFactorSources::new([], 1, [fs.clone()]),
        Profile::new([fs], [&a], []),
        IndexSet::new(), // <---- EMPTY => should_panic
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();
}

#[should_panic]
#[actix_rt::test]
async fn mfa_panics_if_entity_unknown() {
    let fs = HDFactorSource::fs0();
    let a = Account::sample_unsecurified();
    let _ = Sut::for_account_mfa(
        &mut FactorInstancesCache::default(),
        MatrixOfFactorSources::new([], 1, [fs.clone()]),
        Profile::new([fs], [&a], []),
        IndexSet::just(Account::a1().entity_address()), // <---- unknown => should_panic
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();
}

#[should_panic]
#[actix_rt::test]
async fn mfa_panics_if_wrong_network() {
    let fs = HDFactorSource::fs0();
    let network = NetworkID::Mainnet;
    let mainnet_account = Account::unsecurified_on_network(
        "main",
        network,
        HierarchicalDeterministicFactorInstance::tx_on_network(
            CAP26EntityKind::Account,
            network,
            HDPathComponent::unsecurified_hardening_base_index(0),
            fs.factor_source_id(),
        ),
    );
    let network = NetworkID::Stokenet;
    let stokenet_account = Account::unsecurified_on_network(
        "stoknet",
        network,
        HierarchicalDeterministicFactorInstance::tx_on_network(
            CAP26EntityKind::Account,
            network,
            HDPathComponent::unsecurified_hardening_base_index(0),
            fs.factor_source_id(),
        ),
    );
    let profile = Profile::new([fs.clone()], [&mainnet_account, &stokenet_account], []);
    assert_eq!(profile.networks.len(), 2);
    let _ = Sut::for_account_mfa(
        &mut FactorInstancesCache::default(),
        MatrixOfFactorSources::new([], 1, [fs.clone()]),
        profile,
        IndexSet::from_iter([
            mainnet_account.entity_address(),
            stokenet_account.entity_address(),
        ]), // <---- wrong network => should_panic
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();
}

#[actix_rt::test]
async fn create_accounts_when_last_is_used_cache_is_fill_only_with_account_vecis_and_if_profile_is_used_a_new_account_is_created(
) {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;
    for i in 0..CACHE_FILLING_QUANTITY {
        let name = format!("Acco {}", i);
        let (acco, stats) = os
            .new_mainnet_account_with_bdfs(name.clone())
            .await
            .unwrap();
        assert_eq!(acco.name, name);
        assert_eq!(stats.debug_was_cached.len(), 0);
        assert_eq!(stats.debug_was_derived.len(), 0);
    }
    assert_eq!(
        os.profile_snapshot().get_accounts().len(),
        CACHE_FILLING_QUANTITY
    );

    let (acco, stats) = os
        .new_mainnet_account_with_bdfs("newly derive")
        .await
        .unwrap();

    assert_eq!(
        os.profile_snapshot().get_accounts().len(),
        CACHE_FILLING_QUANTITY + 1
    );

    assert_eq!(stats.debug_was_cached.len(), CACHE_FILLING_QUANTITY);
    assert_eq!(stats.debug_was_derived.len(), CACHE_FILLING_QUANTITY + 1);

    assert_eq!(
        acco.as_unsecurified()
            .unwrap()
            .factor_instance()
            .derivation_entity_index(),
        HDPathComponent::unsecurified_hardening_base_index(30)
    );
    assert!(os
        .cache_snapshot()
        .is_full(NetworkID::Mainnet, bdfs.factor_source_id()));

    // and another one
    let (acco, stats) = os
        .new_mainnet_account_with_bdfs("newly derive 2")
        .await
        .unwrap();

    assert_eq!(
        os.profile_snapshot().get_accounts().len(),
        CACHE_FILLING_QUANTITY + 2
    );

    assert_eq!(stats.debug_was_cached.len(), 0);
    assert_eq!(stats.debug_was_derived.len(), 0);

    assert_eq!(
        acco.as_unsecurified()
            .unwrap()
            .factor_instance()
            .derivation_entity_index(),
        HDPathComponent::unsecurified_hardening_base_index(31)
    );
    assert!(
        !os.cache_snapshot()
            .is_full(NetworkID::Mainnet, bdfs.factor_source_id()),
        "just consumed one, so not full"
    );
}

#[actix_rt::test]
async fn cache_is_always_filled_persona_veci_then_after_all_used_we_start_over_at_zero_if_no_profile_is_used(
) {
    let network = NetworkID::Mainnet;
    let bdfs = HDFactorSource::sample();
    let mut cache = FactorInstancesCache::default();

    let outcome = Sut::for_persona_veci(
        &mut cache,
        None,
        bdfs.clone(),
        network,
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();

    assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

    assert_eq!(outcome.debug_found_in_cache.len(), 0);

    assert_eq!(
        outcome.debug_was_cached.len(),
        DerivationPreset::all().len() * CACHE_FILLING_QUANTITY
    );

    assert_eq!(
        outcome.debug_was_derived.len(),
        DerivationPreset::all().len() * CACHE_FILLING_QUANTITY + 1
    );

    let instances_used_directly = outcome.to_use_directly.factor_instances();
    assert_eq!(instances_used_directly.len(), 1);
    let instances_used_directly = instances_used_directly.first().unwrap();

    assert_eq!(
        instances_used_directly.derivation_entity_index(),
        HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
            UnsecurifiedIndex::unsecurified_hardening_base_index(0)
        ))
    );

    cache.assert_is_full(network, bdfs.factor_source_id());

    let cached = cache
        .peek_all_instances_of_factor_source(bdfs.factor_source_id())
        .unwrap();

    let persona_veci_paths = cached
        .clone()
        .get(&DerivationPreset::IdentityVeci.index_agnostic_path_on_network(network))
        .unwrap()
        .factor_instances()
        .into_iter()
        .map(|x| x.derivation_path())
        .collect_vec();

    assert_eq!(persona_veci_paths.len(), CACHE_FILLING_QUANTITY);

    assert!(persona_veci_paths
        .iter()
        .all(|x| x.entity_kind == CAP26EntityKind::Identity
            && x.network_id == network
            && x.key_space() == KeySpace::Unsecurified
            && x.key_kind == CAP26KeyKind::TransactionSigning));

    let persona_veci_indices = persona_veci_paths
        .into_iter()
        .map(|x| x.index)
        .collect_vec();

    assert_eq!(
        persona_veci_indices.first().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(1)
    );

    assert_eq!(
        persona_veci_indices.last().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(30)
    );
}

#[actix_rt::test]
async fn cache_is_always_filled_account_veci_then_after_all_used_we_start_over_at_zero_if_no_profile_is_used(
) {
    let network = NetworkID::Mainnet;
    let bdfs = HDFactorSource::sample();
    let mut cache = FactorInstancesCache::default();

    let outcome = Sut::for_account_veci(
        &mut cache,
        None,
        bdfs.clone(),
        network,
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();

    assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

    assert_eq!(outcome.debug_found_in_cache.len(), 0);

    assert_eq!(
        outcome.debug_was_cached.len(),
        DerivationPreset::all().len() * CACHE_FILLING_QUANTITY
    );

    assert_eq!(
        outcome.debug_was_derived.len(),
        DerivationPreset::all().len() * CACHE_FILLING_QUANTITY + 1
    );

    let instances_used_directly = outcome.to_use_directly.factor_instances();
    assert_eq!(instances_used_directly.len(), 1);
    let instances_used_directly = instances_used_directly.first().unwrap();

    assert_eq!(
        instances_used_directly.derivation_entity_index(),
        HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
            UnsecurifiedIndex::unsecurified_hardening_base_index(0)
        ))
    );

    cache.assert_is_full(network, bdfs.factor_source_id());

    let cached = cache
        .peek_all_instances_of_factor_source(bdfs.factor_source_id())
        .unwrap();

    let account_veci_paths = cached
        .clone()
        .get(&DerivationPreset::AccountVeci.index_agnostic_path_on_network(network))
        .unwrap()
        .factor_instances()
        .into_iter()
        .map(|x| x.derivation_path())
        .collect_vec();

    assert_eq!(account_veci_paths.len(), CACHE_FILLING_QUANTITY);

    assert!(account_veci_paths
        .iter()
        .all(|x| x.entity_kind == CAP26EntityKind::Account
            && x.network_id == network
            && x.key_space() == KeySpace::Unsecurified
            && x.key_kind == CAP26KeyKind::TransactionSigning));

    let account_veci_indices = account_veci_paths
        .into_iter()
        .map(|x| x.index)
        .collect_vec();

    assert_eq!(
        account_veci_indices.first().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(1)
    );

    assert_eq!(
        account_veci_indices.last().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(30)
    );

    let account_mfa_paths = cached
        .clone()
        .get(&DerivationPreset::AccountMfa.index_agnostic_path_on_network(network))
        .unwrap()
        .factor_instances()
        .into_iter()
        .map(|x| x.derivation_path())
        .collect_vec();

    assert!(account_mfa_paths
        .iter()
        .all(|x| x.entity_kind == CAP26EntityKind::Account
            && x.network_id == network
            && x.key_space() == KeySpace::Securified
            && x.key_kind == CAP26KeyKind::TransactionSigning));

    let account_mfa_indices = account_mfa_paths.into_iter().map(|x| x.index).collect_vec();

    assert_eq!(
        account_mfa_indices.first().unwrap().clone(),
        HDPathComponent::securifying_base_index(0)
    );

    assert_eq!(
        account_mfa_indices.last().unwrap().clone(),
        HDPathComponent::securifying_base_index(29)
    );

    let identity_mfa_paths = cached
        .clone()
        .get(&DerivationPreset::IdentityMfa.index_agnostic_path_on_network(network))
        .unwrap()
        .factor_instances()
        .into_iter()
        .map(|x| x.derivation_path())
        .collect_vec();

    assert!(identity_mfa_paths
        .iter()
        .all(|x| x.entity_kind == CAP26EntityKind::Identity
            && x.network_id == network
            && x.key_space() == KeySpace::Securified
            && x.key_kind == CAP26KeyKind::TransactionSigning));

    let identity_mfa_indices = identity_mfa_paths
        .into_iter()
        .map(|x| x.index)
        .collect_vec();

    assert_eq!(
        identity_mfa_indices.first().unwrap().clone(),
        HDPathComponent::securifying_base_index(0)
    );

    assert_eq!(
        identity_mfa_indices.last().unwrap().clone(),
        HDPathComponent::securifying_base_index(29)
    );

    let identity_veci_paths = cached
        .clone()
        .get(&DerivationPreset::IdentityVeci.index_agnostic_path_on_network(network))
        .unwrap()
        .factor_instances()
        .into_iter()
        .map(|x| x.derivation_path())
        .collect_vec();

    assert!(identity_veci_paths
        .iter()
        .all(|x| x.entity_kind == CAP26EntityKind::Identity
            && x.network_id == network
            && x.key_space() == KeySpace::Unsecurified
            && x.key_kind == CAP26KeyKind::TransactionSigning));

    let identity_veci_indices = identity_veci_paths
        .into_iter()
        .map(|x| x.index)
        .collect_vec();

    assert_eq!(
        identity_veci_indices.first().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(0)
    );

    assert_eq!(
        identity_veci_indices.last().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(29)
    );

    // lets create another account (same network, same factor source)

    let outcome = Sut::for_account_veci(
        &mut cache,
        None,
        bdfs.clone(),
        network,
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();

    assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());
    assert_eq!(outcome.debug_found_in_cache.len(), 1); // This time we found in cache
    assert_eq!(outcome.debug_was_cached.len(), 0);
    assert_eq!(outcome.debug_was_derived.len(), 0);

    let instances_used_directly = outcome.to_use_directly.factor_instances();
    assert_eq!(instances_used_directly.len(), 1);
    let instances_used_directly = instances_used_directly.first().unwrap();

    assert_eq!(
        instances_used_directly.derivation_entity_index(),
        HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
            UnsecurifiedIndex::unsecurified_hardening_base_index(1) // Next one!
        ))
    );

    assert!(!cache.is_full(network, bdfs.factor_source_id())); // not full anymore, since we just used a veci

    let cached = cache
        .peek_all_instances_of_factor_source(bdfs.factor_source_id())
        .unwrap();

    let account_veci_paths = cached
        .clone()
        .get(&DerivationPreset::AccountVeci.index_agnostic_path_on_network(network))
        .unwrap()
        .factor_instances()
        .into_iter()
        .map(|x| x.derivation_path())
        .collect_vec();

    assert_eq!(account_veci_paths.len(), CACHE_FILLING_QUANTITY - 1);

    assert!(account_veci_paths
        .iter()
        .all(|x| x.entity_kind == CAP26EntityKind::Account
            && x.network_id == network
            && x.key_space() == KeySpace::Unsecurified
            && x.key_kind == CAP26KeyKind::TransactionSigning));

    let account_veci_indices = account_veci_paths
        .into_iter()
        .map(|x| x.index)
        .collect_vec();

    assert_eq!(
        account_veci_indices.first().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(2) // first is not `1` anymore
    );

    assert_eq!(
        account_veci_indices.last().unwrap().clone(),
        HDPathComponent::unsecurified_hardening_base_index(30)
    );

    // create 29 more accounts, then we should be able to crate one more which should ONLY derive
    // more instances for ACCOUNT VECI, and not Identity Veci, Identity MFA and Account MFA, since that is
    // not needed.
    for _ in 0..29 {
        let outcome = Sut::for_account_veci(
            &mut cache,
            None,
            bdfs.clone(),
            network,
            Arc::new(TestDerivationInteractors::default()),
        )
        .await
        .unwrap();

        assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

        assert_eq!(outcome.debug_found_in_cache.len(), 1);
        assert_eq!(outcome.debug_was_cached.len(), 0);
        assert_eq!(outcome.debug_was_derived.len(), 0);
    }

    let cached = cache
        .peek_all_instances_of_factor_source(bdfs.factor_source_id())
        .unwrap();

    assert!(
        cached
            .get(&DerivationPreset::AccountVeci.index_agnostic_path_on_network(network))
            .is_none(),
        "should have used the last instance..."
    );

    // Great, now lets create one more account, and this time we should derive more instances for
    // it. We should derive 31 instances, 30 for account veci to cache and 1 to use directly.
    // we should NOT derive more instances for Identity Veci, Identity MFA and Account MFA, since
    // that cache is already full.
    let outcome = Sut::for_account_veci(
        &mut cache,
        None,
        bdfs.clone(),
        network,
        Arc::new(TestDerivationInteractors::default()),
    )
    .await
    .unwrap();

    assert_eq!(outcome.factor_source_id, bdfs.factor_source_id());

    assert_eq!(outcome.debug_found_in_cache.len(), 0);
    assert_eq!(outcome.debug_was_cached.len(), CACHE_FILLING_QUANTITY); // ONLY 30, not 120...
    assert_eq!(outcome.debug_was_derived.len(), CACHE_FILLING_QUANTITY + 1);

    let instances_used_directly = outcome.to_use_directly.factor_instances();
    assert_eq!(instances_used_directly.len(), 1);
    let instances_used_directly = instances_used_directly.first().unwrap();

    assert_eq!(
        instances_used_directly.derivation_entity_index(),
        HDPathComponent::Hardened(HDPathComponentHardened::Unsecurified(
            UnsecurifiedIndex::unsecurified_hardening_base_index(0) // IMPORTANT! Index 0 is used again! Why?! Well because are not using a Profile here, and we are not eagerly filling cache just before we are using the last index.
        ))
    );
}

#[actix_rt::test]
async fn add_factor_source() {
    let mut os = SargonOS::new();
    assert_eq!(os.cache_snapshot().total_number_of_factor_instances(), 0);
    assert_eq!(os.profile_snapshot().factor_sources.len(), 0);
    let factor_source = HDFactorSource::sample();
    os.add_factor_source(factor_source.clone()).await.unwrap();
    assert!(
        os.cache_snapshot()
            .is_full(NetworkID::Mainnet, factor_source.factor_source_id()),
        "Should have put factors into the cache."
    );
    assert_eq!(
        os.profile_snapshot().factor_sources,
        IndexSet::just(factor_source)
    );
}

#[actix_rt::test]
async fn adding_accounts_and_clearing_cache_in_between() {
    let (mut os, _) = SargonOS::with_bdfs().await;
    assert!(os.profile_snapshot().get_accounts().is_empty());
    let (alice, stats) = os.new_mainnet_account_with_bdfs("alice").await.unwrap();
    assert!(!stats.debug_found_in_cache.is_empty());
    assert!(stats.debug_was_cached.is_empty());
    assert!(stats.debug_was_derived.is_empty());
    os.clear_cache();

    let (bob, stats) = os.new_mainnet_account_with_bdfs("bob").await.unwrap();
    assert!(stats.debug_found_in_cache.is_empty());
    assert!(!stats.debug_was_cached.is_empty());
    assert!(!stats.debug_was_derived.is_empty());
    assert_ne!(alice, bob);

    assert_eq!(os.profile_snapshot().get_accounts().len(), 2);
}

#[actix_rt::test]
async fn adding_personas_and_clearing_cache_in_between() {
    let (mut os, _) = SargonOS::with_bdfs().await;
    assert!(os.profile_snapshot().get_personas().is_empty());
    let (batman, stats) = os.new_mainnet_persona_with_bdfs("Batman").await.unwrap();

    assert_eq!(
        batman
            .clone()
            .security_state
            .into_unsecured()
            .unwrap()
            .derivation_path()
            .entity_kind,
        CAP26EntityKind::Identity
    );
    assert!(!stats.debug_found_in_cache.is_empty());
    assert!(stats.debug_was_cached.is_empty());
    assert!(stats.debug_was_derived.is_empty());
    os.clear_cache();

    let (satoshi, stats) = os.new_mainnet_persona_with_bdfs("Satoshi").await.unwrap();
    assert!(stats.debug_found_in_cache.is_empty());
    assert!(!stats.debug_was_cached.is_empty());
    assert!(!stats.debug_was_derived.is_empty());
    assert_ne!(batman, satoshi);

    assert_eq!(os.profile_snapshot().get_personas().len(), 2);
}

#[actix_rt::test]
async fn add_account_and_personas_mixed() {
    let (mut os, _) = SargonOS::with_bdfs().await;
    assert!(os.profile_snapshot().get_personas().is_empty());
    assert!(os.profile_snapshot().get_accounts().is_empty());

    let (batman, stats) = os.new_mainnet_persona_with_bdfs("Batman").await.unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (alice, stats) = os.new_mainnet_account_with_bdfs("alice").await.unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (satoshi, stats) = os.new_mainnet_persona_with_bdfs("Satoshi").await.unwrap();
    assert!(stats.debug_was_derived.is_empty());

    assert_ne!(batman.entity_address(), satoshi.entity_address());

    let (bob, stats) = os.new_mainnet_account_with_bdfs("bob").await.unwrap();
    assert!(stats.debug_was_derived.is_empty());
    assert_ne!(alice.entity_address(), bob.entity_address());

    assert_eq!(os.profile_snapshot().get_personas().len(), 2);
    assert_eq!(os.profile_snapshot().get_accounts().len(), 2);
}

#[actix_rt::test]
async fn adding_accounts_different_networks_different_factor_sources() {
    let mut os = SargonOS::new();
    assert_eq!(os.cache_snapshot().total_number_of_factor_instances(), 0);

    let fs_device = HDFactorSource::device();
    let fs_arculus = HDFactorSource::arculus();
    let fs_ledger = HDFactorSource::ledger();

    os.add_factor_source(fs_device.clone()).await.unwrap();
    os.add_factor_source(fs_arculus.clone()).await.unwrap();
    os.add_factor_source(fs_ledger.clone()).await.unwrap();

    assert_eq!(
        os.cache_snapshot().total_number_of_factor_instances(),
        3 * 4 * CACHE_FILLING_QUANTITY
    );

    assert!(os.profile_snapshot().get_accounts().is_empty());
    assert_eq!(os.profile_snapshot().factor_sources.len(), 3);

    let (alice, stats) = os
        .new_account(fs_device.clone(), NetworkID::Mainnet, "Alice")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (bob, stats) = os
        .new_account(fs_device.clone(), NetworkID::Mainnet, "Bob")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (carol, stats) = os
        .new_account(fs_device.clone(), NetworkID::Stokenet, "Carol")
        .await
        .unwrap();
    assert!(
        !stats.debug_was_derived.is_empty(),
        "Should have derived more, since first time Stokenet is used!"
    );

    let (diana, stats) = os
        .new_account(fs_device.clone(), NetworkID::Stokenet, "Diana")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (erin, stats) = os
        .new_account(fs_arculus.clone(), NetworkID::Mainnet, "Erin")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (frank, stats) = os
        .new_account(fs_arculus.clone(), NetworkID::Mainnet, "Frank")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (grace, stats) = os
        .new_account(fs_arculus.clone(), NetworkID::Stokenet, "Grace")
        .await
        .unwrap();
    assert!(
        !stats.debug_was_derived.is_empty(),
        "Should have derived more, since first time Stokenet is used with the Arculus!"
    );

    let (helena, stats) = os
        .new_account(fs_arculus.clone(), NetworkID::Stokenet, "Helena")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (isabel, stats) = os
        .new_account(fs_ledger.clone(), NetworkID::Mainnet, "isabel")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (jenny, stats) = os
        .new_account(fs_ledger.clone(), NetworkID::Mainnet, "Jenny")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (klara, stats) = os
        .new_account(fs_ledger.clone(), NetworkID::Stokenet, "Klara")
        .await
        .unwrap();
    assert!(
        !stats.debug_was_derived.is_empty(),
        "Should have derived more, since first time Stokenet is used with the Ledger!"
    );

    let (lisa, stats) = os
        .new_account(fs_ledger.clone(), NetworkID::Stokenet, "Lisa")
        .await
        .unwrap();
    assert!(stats.debug_was_derived.is_empty());

    assert_eq!(os.profile_snapshot().get_accounts().len(), 12);

    let accounts = vec![
        alice, bob, carol, diana, erin, frank, grace, helena, isabel, jenny, klara, lisa,
    ];

    let factor_source_count = os.profile_snapshot().factor_sources.len();
    let network_count = os.profile_snapshot().networks.len();
    assert_eq!(
        os.cache_snapshot().total_number_of_factor_instances(),
        network_count
            * factor_source_count
            * DerivationPreset::all().len()
            * CACHE_FILLING_QUANTITY
            - accounts.len()
            + factor_source_count // we do `+ factor_source_count` since every time a factor source is used on a new network for the first time, we derive `CACHE_FILLING_QUANTITY + 1`
    );

    assert_eq!(
        os.profile_snapshot()
            .get_accounts()
            .into_iter()
            .map(|a| a.entity_address())
            .collect::<HashSet<AccountAddress>>(),
        accounts
            .into_iter()
            .map(|a| a.entity_address())
            .collect::<HashSet<AccountAddress>>()
    );
}

#[actix_rt::test]
async fn securify_accounts_and_personas_with_override_factor() {
    // this is mostly a soundness test for the two functions `for_persona_mfa` and `for_account_mfa`
    // using `os` because I'm lazy. We might in fact remove `for_persona_mfa` and `for_account_mfa`
    // and only use the `for_entity_mfa` function... but we have these to get code coverage.
    let (mut os, bdfs) = SargonOS::with_bdfs().await;

    let (batman, stats) = os.new_mainnet_persona_with_bdfs("Batman").await.unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let (alice, stats) = os.new_mainnet_account_with_bdfs("alice").await.unwrap();
    assert!(stats.debug_was_derived.is_empty());

    let shield_0 = MatrixOfFactorSources::new([], 0, [bdfs.clone()]);
    let mut cache = os.cache_snapshot();
    let interactors = Arc::new(TestDerivationInteractors::default());
    let outcome = FactorInstancesProvider::for_account_mfa(
        &mut cache,
        shield_0.clone(),
        os.profile_snapshot(),
        IndexSet::just(alice.entity_address()),
        interactors.clone(),
    )
    .await
    .unwrap();
    let outcome = outcome.per_factor.get(&bdfs.factor_source_id()).unwrap();
    assert_eq!(outcome.to_use_directly.len(), 1);

    let outcome = FactorInstancesProvider::for_persona_mfa(
        &mut cache,
        shield_0.clone(),
        os.profile_snapshot(),
        IndexSet::just(batman.entity_address()),
        interactors.clone(),
    )
    .await
    .unwrap();
    let outcome = outcome.per_factor.get(&bdfs.factor_source_id()).unwrap();
    assert_eq!(outcome.to_use_directly.len(), 1);
}

#[actix_rt::test]
async fn test_securified_accounts() {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;
    let alice = os
        .new_account_with_bdfs(NetworkID::Mainnet, "Alice")
        .await
        .unwrap()
        .0;

    let bob = os
        .new_account_with_bdfs(NetworkID::Mainnet, "Bob")
        .await
        .unwrap()
        .0;
    assert_ne!(alice.address(), bob.address());
    let ledger = HDFactorSource::ledger();
    let arculus = HDFactorSource::arculus();
    let yubikey = HDFactorSource::yubikey();
    os.add_factor_source(ledger.clone()).await.unwrap();
    os.add_factor_source(arculus.clone()).await.unwrap();
    os.add_factor_source(yubikey.clone()).await.unwrap();
    let shield_0 =
        MatrixOfFactorSources::new([bdfs.clone(), ledger.clone(), arculus.clone()], 2, []);

    let (securified_accounts, stats) = os
        .securify_accounts(
            IndexSet::from_iter([alice.entity_address(), bob.entity_address()]),
            shield_0,
        )
        .await
        .unwrap();

    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );

    let alice_sec = securified_accounts
        .clone()
        .into_iter()
        .find(|x| x.address() == alice.entity_address())
        .unwrap();

    assert_eq!(
        alice_sec.securified_entity_control().veci.unwrap().clone(),
        alice.as_unsecurified().unwrap().veci().factor_instance()
    );
    let alice_matrix = alice_sec.securified_entity_control().matrix.clone();
    assert_eq!(alice_matrix.threshold, 2);

    assert_eq!(
        alice_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec(),
        [
            bdfs.factor_source_id(),
            ledger.factor_source_id(),
            arculus.factor_source_id()
        ]
    );

    assert_eq!(
        alice_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [
            HDPathComponent::securifying_base_index(0),
            HDPathComponent::securifying_base_index(0),
            HDPathComponent::securifying_base_index(0)
        ]
    );

    // assert bob

    let bob_sec = securified_accounts
        .clone()
        .into_iter()
        .find(|x| x.address() == bob.entity_address())
        .unwrap();

    assert_eq!(
        bob_sec.securified_entity_control().veci.unwrap().clone(),
        bob.as_unsecurified().unwrap().veci().factor_instance()
    );
    let bob_matrix = bob_sec.securified_entity_control().matrix.clone();
    assert_eq!(bob_matrix.threshold, 2);

    assert_eq!(
        bob_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec(),
        [
            bdfs.factor_source_id(),
            ledger.factor_source_id(),
            arculus.factor_source_id()
        ]
    );

    assert_eq!(
        bob_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [
            HDPathComponent::securifying_base_index(1),
            HDPathComponent::securifying_base_index(1),
            HDPathComponent::securifying_base_index(1)
        ]
    );

    let carol = os
        .new_account(ledger.clone(), NetworkID::Mainnet, "Carol")
        .await
        .unwrap()
        .0;

    assert_eq!(
            carol
                .as_unsecurified()
                .unwrap()
                .veci()
                .factor_instance()
                .derivation_entity_index()
                .base_index(),
            0,
            "First account created with ledger, should have index 0, even though this ledger was used in the shield, since we are using two different KeySpaces for Securified and Unsecurified accounts."
        );

    let (securified_accounts, stats) = os
        .securify_accounts(
            IndexSet::just(carol.entity_address()),
            MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
        )
        .await
        .unwrap();
    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );
    let carol_sec = securified_accounts
        .clone()
        .into_iter()
        .find(|x| x.address() == carol.entity_address())
        .unwrap();

    let carol_matrix = carol_sec.securified_entity_control().matrix.clone();

    assert_eq!(
        carol_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec(),
        [yubikey.factor_source_id()]
    );

    assert_eq!(
        carol_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [HDPathComponent::securifying_base_index(0)]
    );

    // Update Alice's shield to only use YubiKey

    let (securified_accounts, stats) = os
        .securify_accounts(
            IndexSet::from_iter([alice.entity_address(), bob.entity_address()]),
            MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
        )
        .await
        .unwrap();
    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );
    let alice_sec = securified_accounts
        .clone()
        .into_iter()
        .find(|x| x.address() == alice.entity_address())
        .unwrap();

    let alice_matrix = alice_sec.securified_entity_control().matrix.clone();

    assert_eq!(
        alice_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [
                HDPathComponent::securifying_base_index(1) // Carol used `0`.
            ]
    );
}

#[actix_rt::test]
async fn securify_accounts_when_cache_is_half_full_single_factor_source() {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;

    let factor_sources = os.profile_snapshot().factor_sources.clone();
    assert_eq!(
        factor_sources.clone().into_iter().collect_vec(),
        vec![bdfs.clone(),]
    );

    let n = CACHE_FILLING_QUANTITY / 2;

    for i in 0..3 * n {
        let _ = os
            .new_mainnet_account_with_bdfs(format!("Acco: {}", i))
            .await
            .unwrap();
    }

    let shield_0 = MatrixOfFactorSources::new([bdfs.clone()], 1, []);

    let all_accounts = os
        .profile_snapshot()
        .get_accounts()
        .into_iter()
        .collect_vec();

    let first_half_of_accounts = all_accounts.clone()[0..n]
        .iter()
        .cloned()
        .collect::<IndexSet<Account>>();

    let second_half_of_accounts = all_accounts.clone()[n..3 * n]
        .iter()
        .cloned()
        .collect::<IndexSet<Account>>();

    assert_eq!(
        first_half_of_accounts.len() + second_half_of_accounts.len(),
        3 * n
    );

    let (first_half_securified_accounts, stats) = os
        .securify_accounts(
            first_half_of_accounts
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            shield_0.clone(),
        )
        .await
        .unwrap();

    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );

    assert_eq!(
        first_half_securified_accounts
            .into_iter()
            .map(|a| a
                .securified_entity_control()
                .primary_role_instances()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .map(|x| format!("{:?}", x))
                .next()
                .unwrap()) // single factor per role text
            .collect_vec(),
        [
            "0^", "1^", "2^", "3^", "4^", "5^", "6^", "7^", "8^", "9^", "10^", "11^", "12^", "13^",
            "14^"
        ]
    );

    let (second_half_securified_accounts, stats) = os
        .securify_accounts(
            second_half_of_accounts
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            shield_0,
        )
        .await
        .unwrap();

    assert!(
        stats.derived_any_new_instance_for_any_factor_source(),
        "should have derived more"
    );

    assert_eq!(
        second_half_securified_accounts
            .into_iter()
            .map(|a| a
                .securified_entity_control()
                .primary_role_instances()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .map(|x| format!("{:?}", x))
                .next()
                .unwrap()) // single factor per role text
            .collect_vec(),
        [
            "15^", "16^", "17^", "18^", "19^", "20^", "21^", "22^", "23^", "24^", "25^", "26^",
            "27^", "28^", "29^", "30^", "31^", "32^", "33^", "34^", "35^", "36^", "37^", "38^",
            "39^", "40^", "41^", "42^", "43^", "44^"
        ]
    );
}

#[actix_rt::test]
async fn securify_accounts_when_cache_is_half_full_multiple_factor_sources() {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;

    let ledger = HDFactorSource::ledger();
    let arculus = HDFactorSource::arculus();
    let yubikey = HDFactorSource::yubikey();
    os.add_factor_source(ledger.clone()).await.unwrap();
    os.add_factor_source(arculus.clone()).await.unwrap();
    os.add_factor_source(yubikey.clone()).await.unwrap();

    let factor_sources = os.profile_snapshot().factor_sources.clone();
    assert_eq!(
        factor_sources.clone().into_iter().collect_vec(),
        vec![
            bdfs.clone(),
            ledger.clone(),
            arculus.clone(),
            yubikey.clone(),
        ]
    );

    let n = CACHE_FILLING_QUANTITY / 2;

    for i in 0..3 * n {
        let (_account, _stats) = os
            .new_mainnet_account_with_bdfs(format!("Acco: {}", i))
            .await
            .unwrap();
    }

    let shield_0 =
        MatrixOfFactorSources::new([bdfs.clone(), ledger.clone(), arculus.clone()], 2, []);

    let all_accounts = os
        .profile_snapshot()
        .get_accounts()
        .into_iter()
        .collect_vec();

    let first_half_of_accounts = all_accounts.clone()[0..n]
        .iter()
        .cloned()
        .collect::<IndexSet<Account>>();

    let second_half_of_accounts = all_accounts.clone()[n..3 * n]
        .iter()
        .cloned()
        .collect::<IndexSet<Account>>();

    assert_eq!(
        first_half_of_accounts.len() + second_half_of_accounts.len(),
        3 * n
    );

    let (first_half_securified_accounts, stats) = os
        .securify_accounts(
            first_half_of_accounts
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            shield_0.clone(),
        )
        .await
        .unwrap();

    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );

    assert_eq!(
        first_half_securified_accounts
            .into_iter()
            .map(|a| a
                .securified_entity_control()
                .primary_role_instances()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .map(|x| format!("{:?}", x))
                .collect_vec())
            .collect_vec(),
        [
            ["0^", "0^", "0^"],
            ["1^", "1^", "1^"],
            ["2^", "2^", "2^"],
            ["3^", "3^", "3^"],
            ["4^", "4^", "4^"],
            ["5^", "5^", "5^"],
            ["6^", "6^", "6^"],
            ["7^", "7^", "7^"],
            ["8^", "8^", "8^"],
            ["9^", "9^", "9^"],
            ["10^", "10^", "10^"],
            ["11^", "11^", "11^"],
            ["12^", "12^", "12^"],
            ["13^", "13^", "13^"],
            ["14^", "14^", "14^"]
        ]
    );

    let (second_half_securified_accounts, stats) = os
        .securify_accounts(
            second_half_of_accounts
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            shield_0,
        )
        .await
        .unwrap();

    assert!(
        stats.derived_any_new_instance_for_any_factor_source(),
        "should have derived more"
    );

    assert!(
        stats.found_any_instances_in_cache_for_any_factor_source(),
        "should have found some in cache"
    );

    assert_eq!(
        second_half_securified_accounts
            .into_iter()
            .map(|a| a
                .securified_entity_control()
                .primary_role_instances()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .map(|x| format!("{:?}", x))
                .collect_vec())
            .collect_vec(),
        [
            ["15^", "15^", "15^"],
            ["16^", "16^", "16^"],
            ["17^", "17^", "17^"],
            ["18^", "18^", "18^"],
            ["19^", "19^", "19^"],
            ["20^", "20^", "20^"],
            ["21^", "21^", "21^"],
            ["22^", "22^", "22^"],
            ["23^", "23^", "23^"],
            ["24^", "24^", "24^"],
            ["25^", "25^", "25^"],
            ["26^", "26^", "26^"],
            ["27^", "27^", "27^"],
            ["28^", "28^", "28^"],
            ["29^", "29^", "29^"],
            ["30^", "30^", "30^"],
            ["31^", "31^", "31^"],
            ["32^", "32^", "32^"],
            ["33^", "33^", "33^"],
            ["34^", "34^", "34^"],
            ["35^", "35^", "35^"],
            ["36^", "36^", "36^"],
            ["37^", "37^", "37^"],
            ["38^", "38^", "38^"],
            ["39^", "39^", "39^"],
            ["40^", "40^", "40^"],
            ["41^", "41^", "41^"],
            ["42^", "42^", "42^"],
            ["43^", "43^", "43^"],
            ["44^", "44^", "44^"]
        ]
    );
}

#[actix_rt::test]
async fn securify_personas_when_cache_is_half_full_single_factor_source() {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;

    let factor_sources = os.profile_snapshot().factor_sources.clone();
    assert_eq!(
        factor_sources.clone().into_iter().collect_vec(),
        vec![bdfs.clone(),]
    );

    let n = CACHE_FILLING_QUANTITY / 2;

    for i in 0..3 * n {
        let _ = os
            .new_mainnet_persona_with_bdfs(format!("Persona: {}", i))
            .await
            .unwrap();
    }

    let shield_0 = MatrixOfFactorSources::new([bdfs.clone()], 1, []);

    let all_personas = os
        .profile_snapshot()
        .get_personas()
        .into_iter()
        .collect_vec();

    let first_half_of_personas = all_personas.clone()[0..n]
        .iter()
        .cloned()
        .collect::<IndexSet<Persona>>();

    let second_half_of_personas = all_personas.clone()[n..3 * n]
        .iter()
        .cloned()
        .collect::<IndexSet<Persona>>();

    assert_eq!(
        first_half_of_personas.len() + second_half_of_personas.len(),
        3 * n
    );

    let (first_half_securified_personas, stats) = os
        .securify_personas(
            first_half_of_personas
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            shield_0.clone(),
        )
        .await
        .unwrap();

    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );

    assert_eq!(
        first_half_securified_personas
            .into_iter()
            .map(|a| a
                .securified_entity_control()
                .primary_role_instances()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .map(|x| format!("{:?}", x))
                .next()
                .unwrap()) // single factor per role text
            .collect_vec(),
        [
            "0^", "1^", "2^", "3^", "4^", "5^", "6^", "7^", "8^", "9^", "10^", "11^", "12^", "13^",
            "14^"
        ]
    );

    let (second_half_securified_personas, stats) = os
        .securify_personas(
            second_half_of_personas
                .clone()
                .into_iter()
                .map(|a| a.entity_address())
                .collect(),
            shield_0,
        )
        .await
        .unwrap();

    assert!(
        stats.derived_any_new_instance_for_any_factor_source(),
        "should have derived more"
    );

    assert_eq!(
        second_half_securified_personas
            .into_iter()
            .map(|a| a
                .securified_entity_control()
                .primary_role_instances()
                .into_iter()
                .map(|f| f.derivation_entity_index())
                .map(|x| format!("{:?}", x))
                .next()
                .unwrap()) // single factor per role text
            .collect_vec(),
        [
            "15^", "16^", "17^", "18^", "19^", "20^", "21^", "22^", "23^", "24^", "25^", "26^",
            "27^", "28^", "29^", "30^", "31^", "32^", "33^", "34^", "35^", "36^", "37^", "38^",
            "39^", "40^", "41^", "42^", "43^", "44^"
        ]
    );
}

#[actix_rt::test]
async fn create_single_account() {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;
    let (alice, stats) = os.new_mainnet_account_with_bdfs("alice").await.unwrap();
    assert!(stats.debug_was_derived.is_empty(), "should have used cache");
    let (sec_accounts, stats) = os
        .securify_accounts(
            IndexSet::just(alice.entity_address()),
            MatrixOfFactorSources::new([], 0, [bdfs]),
        )
        .await
        .unwrap();
    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );
    let alice_sec = sec_accounts.into_iter().next().unwrap();
    assert_eq!(
        alice_sec
            .securified_entity_control()
            .primary_role_instances()
            .first()
            .unwrap()
            .derivation_entity_index(),
        HDPathComponent::securifying_base_index(0)
    );
}

#[actix_rt::test]
async fn securified_personas() {
    let (mut os, bdfs) = SargonOS::with_bdfs().await;
    let batman = os
        .new_persona_with_bdfs(NetworkID::Mainnet, "Batman")
        .await
        .unwrap()
        .0;

    let satoshi = os
        .new_persona_with_bdfs(NetworkID::Mainnet, "Satoshi")
        .await
        .unwrap()
        .0;
    assert_ne!(batman.address(), satoshi.address());
    let ledger = HDFactorSource::ledger();
    let arculus = HDFactorSource::arculus();
    let yubikey = HDFactorSource::yubikey();
    os.add_factor_source(ledger.clone()).await.unwrap();
    os.add_factor_source(arculus.clone()).await.unwrap();
    os.add_factor_source(yubikey.clone()).await.unwrap();
    let shield_0 =
        MatrixOfFactorSources::new([bdfs.clone(), ledger.clone(), arculus.clone()], 2, []);

    let (securified_personas, stats) = os
        .securify_personas(
            IndexSet::from_iter([batman.entity_address(), satoshi.entity_address()]),
            shield_0,
        )
        .await
        .unwrap();

    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );

    let batman_sec = securified_personas
        .clone()
        .into_iter()
        .find(|x| x.address() == batman.entity_address())
        .unwrap();

    assert_eq!(
        batman_sec.securified_entity_control().veci.unwrap().clone(),
        batman.as_unsecurified().unwrap().veci().factor_instance()
    );
    let batman_matrix = batman_sec.securified_entity_control().primary_role();
    assert_eq!(batman_matrix.threshold, 2);

    assert_eq!(
        batman_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec(),
        [
            bdfs.factor_source_id(),
            ledger.factor_source_id(),
            arculus.factor_source_id()
        ]
    );

    assert_eq!(
        batman_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [
            HDPathComponent::securifying_base_index(0),
            HDPathComponent::securifying_base_index(0),
            HDPathComponent::securifying_base_index(0)
        ]
    );

    // assert satoshi

    let satoshi_sec = securified_personas
        .clone()
        .into_iter()
        .find(|x| x.address() == satoshi.entity_address())
        .unwrap();

    assert_eq!(
        satoshi_sec
            .securified_entity_control()
            .veci
            .unwrap()
            .clone(),
        satoshi.as_unsecurified().unwrap().veci().factor_instance()
    );
    let satoshi_matrix = satoshi_sec.securified_entity_control().primary_role();
    assert_eq!(satoshi_matrix.threshold, 2);

    assert_eq!(
        satoshi_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec(),
        [
            bdfs.factor_source_id(),
            ledger.factor_source_id(),
            arculus.factor_source_id()
        ]
    );

    assert_eq!(
        satoshi_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [
            HDPathComponent::securifying_base_index(1),
            HDPathComponent::securifying_base_index(1),
            HDPathComponent::securifying_base_index(1)
        ]
    );

    let hyde = os
        .new_persona(ledger.clone(), NetworkID::Mainnet, "Mr Hyde")
        .await
        .unwrap()
        .0;

    assert_eq!(
        hyde
                .as_unsecurified()
                .unwrap()
                .veci()
                .factor_instance()
                .derivation_entity_index()
                .base_index(),
            0,
            "First persona created with ledger, should have index 0, even though this ledger was used in the shield, since we are using two different KeySpaces for Securified and Unsecurified personas."
        );

    let (securified_personas, stats) = os
        .securify_personas(
            IndexSet::just(hyde.entity_address()),
            MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
        )
        .await
        .unwrap();
    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );
    let hyde_sec = securified_personas
        .clone()
        .into_iter()
        .find(|x| x.address() == hyde.entity_address())
        .unwrap();

    let hyde_matrix = hyde_sec.securified_entity_control().primary_role();

    assert_eq!(
        hyde_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.factor_source_id())
            .collect_vec(),
        [yubikey.factor_source_id()]
    );

    assert_eq!(
        hyde_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [HDPathComponent::securifying_base_index(0)]
    );

    // Update Batmans and Satoshis's shield to only use YubiKey

    let (securified_personas, stats) = os
        .securify_personas(
            IndexSet::from_iter([batman.entity_address(), satoshi.entity_address()]),
            MatrixOfFactorSources::new([], 0, [yubikey.clone()]),
        )
        .await
        .unwrap();
    assert!(
        !stats.derived_any_new_instance_for_any_factor_source(),
        "should have used cache"
    );
    let batman_sec = securified_personas
        .clone()
        .into_iter()
        .find(|x| x.address() == batman.entity_address())
        .unwrap();

    let batman_matrix = batman_sec.securified_entity_control().primary_role();

    assert_eq!(
        batman_matrix
            .all_factors()
            .into_iter()
            .map(|f| f.derivation_entity_index())
            .collect_vec(),
        [
                HDPathComponent::securifying_base_index(1) // Carol used `0`.
            ]
    );
}
