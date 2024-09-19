use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, Hash)]
struct PreDerivedKeysCacheRequestPathTuple {
    request: DerivationRequest,
    path: DerivationPathWithoutIndex,
}
impl PreDerivedKeysCacheRequestPathTuple {
    #[allow(unused)]
    fn cache_key(&self) -> PreDerivedKeysCacheKey {
        PreDerivedKeysCacheKey::new(self.request.factor_source_id, self.path.clone())
    }
}

fn pre_derived_keys_cache_request_path_tuples(
    requests: IndexSet<DerivationRequest>,
) -> IndexSet<PreDerivedKeysCacheRequestPathTuple> {
    requests
        .clone()
        .into_iter()
        .map(|request| PreDerivedKeysCacheRequestPathTuple {
            request,
            path: DerivationPathWithoutIndex::from(request),
        })
        .collect::<IndexSet<PreDerivedKeysCacheRequestPathTuple>>()
}

/// Internal helper for implementing `peek`.
/// `peek` will will this and map:
/// 1. `Err(e)` -> `NextDerivationPeekOutcome::Failure(e)`
/// 1. `Ok(None)` -> `NextDerivationPeekOutcome::Fulfillable`
/// 1. `Ok(requests)` -> `NextDerivationPeekOutcome::Unfulfillable(DerivationRequestsUnfulfillableByCache::new(requests))`
pub fn pre_derived_keys_cache_peek<
    C: Deref<
        Target = HashMap<PreDerivedKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>,
    >,
>(
    requests: IndexSet<DerivationRequest>,
    cached: &C,
) -> NextDerivationPeekOutcome {
    let outcome = pre_derived_keys_cache_try_peek(requests, cached);
    match outcome {
        Ok(None) => NextDerivationPeekOutcome::Fulfillable,
        Ok(Some(unfulfillable)) => NextDerivationPeekOutcome::Unfulfillable(unfulfillable),
        Err(e) => NextDerivationPeekOutcome::Failure(e),
    }
}

fn pre_derived_keys_cache_try_peek<
    C: Deref<
        Target = HashMap<PreDerivedKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>,
    >,
>(
    requests: IndexSet<DerivationRequest>,
    cached: &C,
) -> Result<Option<DerivationRequestsUnfulfillableByCache>> {
    let request_and_path_tuples = pre_derived_keys_cache_request_path_tuples(requests.clone());

    let mut unfulfillable = IndexSet::<DerivationRequestUnfulfillableByCache>::new();
    for tuple in request_and_path_tuples.iter() {
        let request = tuple.request;
        let Some(for_key) = cached.get(&tuple.cache_key()) else {
            unfulfillable.insert(DerivationRequestUnfulfillableByCache::empty(request));
            continue;
        };

        let factors_left = for_key.len();
        if factors_left == 0 {
            unfulfillable.insert(DerivationRequestUnfulfillableByCache::empty(request));
        } else if factors_left == 1 {
            let last_factor = for_key.last().expect("Just checked length.");
            unfulfillable.insert(DerivationRequestUnfulfillableByCache::last(
                request,
                last_factor,
            ));
        } else {
            // all good
            continue;
        }
    }

    if unfulfillable.is_empty() {
        Ok(None)
    } else {
        Ok(Some(DerivationRequestsUnfulfillableByCache::new(
            unfulfillable,
        )))
    }
}

pub fn pre_derived_keys_cache_insert<
    C: DerefMut<
        Target = HashMap<PreDerivedKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>,
    >,
>(
    new: IndexMap<PreDerivedKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>,
    mut_cache: &mut C,
) -> Result<()> {
    let derived_factors_map = new;
    for (key, values) in derived_factors_map.iter() {
        for value in values {
            assert_eq!(
                value.factor_source_id(),
                key.factor_source_id,
                "Discrepancy! FactorSourceID mismatch, this is a developer error."
            );
        }
    }

    for (key, derived_factors) in derived_factors_map {
        if let Some(existing_factors) = mut_cache.get_mut(&key) {
            existing_factors.extend(derived_factors);
        } else {
            mut_cache.insert(key, derived_factors);
        }
    }

    Ok(())
}

pub fn pre_derived_keys_cache_consume<
    C: DerefMut<
        Target = HashMap<PreDerivedKeysCacheKey, IndexSet<HierarchicalDeterministicFactorInstance>>,
    >,
>(
    requests: IndexSet<DerivationRequest>,
    mut_cache: &mut C,
) -> Result<IndexMap<DerivationRequest, HierarchicalDeterministicFactorInstance>> {
    let mut instances_read_from_cache =
        IndexMap::<DerivationRequest, HierarchicalDeterministicFactorInstance>::new();

    let request_and_path_tuples = pre_derived_keys_cache_request_path_tuples(requests.clone());

    for tuple in request_and_path_tuples {
        let for_key = mut_cache
            .get_mut(&tuple.cache_key())
            .ok_or(CommonError::KeysCacheUnknownKey)?;
        let read_from_cache = for_key
            .first()
            .ok_or(CommonError::KeysCacheEmptyForKey)?
            .clone();
        assert!(
            read_from_cache.matches(&tuple.request),
            "incorrect implementation"
        );
        for_key.shift_remove(&read_from_cache);
        instances_read_from_cache.insert(tuple.request, read_from_cache);
    }

    Ok(instances_read_from_cache)
}
