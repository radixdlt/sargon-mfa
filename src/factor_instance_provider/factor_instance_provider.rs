use core::num;

use crate::prelude::*;

/// A cache for pre-derived keys, saved on file and which will derive more keys
/// if needed, using UI/UX via KeysCollector.
#[async_trait::async_trait]
pub trait IsPreDerivedKeysCache {
    /// Must be async since might need to derive more keys if we are about
    /// to use the last, thus will require usage of KeysCollector - which is async.
    /// Also typically we cache to file - which itself is async
    async fn consume_next_factor_instance(
        &self,
        request: DerivationRequest,
    ) -> Result<HierarchicalDeterministicFactorInstance>;

    async fn would_consume_last_factor(&self, request: DerivationRequest) -> bool;
}

/// A simple `IsPreDerivedKeysCache` which uses in-memory cache instead of on
/// file which the live implementation will use.
#[derive(Default)]
pub struct InMemoryPreDerivedKeysCache {
    cache: HashMap<DerivationPath, HierarchicalDeterministicFactorInstance>,
}

#[async_trait::async_trait]
impl IsPreDerivedKeysCache for InMemoryPreDerivedKeysCache {
    async fn consume_next_factor_instance(
        &self,
        request: DerivationRequest,
    ) -> Result<HierarchicalDeterministicFactorInstance> {
        todo!()
    }
    async fn would_consume_last_factor(&self, request: DerivationRequest) -> bool {
        todo!()
    }
}

pub struct FactorInstanceProvider {
    gateway: Arc<dyn Gateway>,
    cache: Arc<dyn IsPreDerivedKeysCache>,
}
impl FactorInstanceProvider {
    pub fn new(gateway: Arc<dyn Gateway>, cache: Arc<dyn IsPreDerivedKeysCache>) -> Self {
        Self { gateway, cache }
    }
}

impl FactorInstanceProvider {
    pub async fn provide_factor_instance<'p>(
        &self,
        request: NextFactorInstanceRequest<'p>,
    ) -> Result<HierarchicalDeterministicFactorInstance> {
        let would_consume_last_factor = self
            .cache
            .would_consume_last_factor(request.derivation_request.clone())
            .await;

        if !would_consume_last_factor {
            return self
                .cache
                .consume_next_factor_instance(request.derivation_request.clone())
                .await;
        }

        // We NEED to have one factor left fulfilling the request, so that we can
        // derive the NEXT keys based on that.
        // This prevents us from a problem:
        // i. Account X with address `A` is created by FactorInstance `F` with { factor_source: L, key_space: Unsecurified, index: 0 }`
        // ii. User securified account X, and `F = { factor_source: L, key_space: Unsecurified, index: 0 }` is now "free", since
        //  it is no longer found in the Profile.
        // iii. User tries to create account Y with `L` and if we would have used Profile "static analysis" it would say that
        // F = { factor_source: L, key_space: Unsecurified, index: 0 }` is next.
        // iv. Failure! Account Y was never created since it would have same address `A` as account X, since it would have used same FactorInstance.
        // v. This problem is we cannot do this simple static analysis of Profile to find next index
        //  we would actually need to form derivation paths and derive the keys and check if that public key
        //  has been used to create any of the addresses in profile.
        //
        // Eureka!: Or we just ensure to not loose track of the fact that `0` has been used, by letting the cache contains
        // (0...N) keys and BEFORE `N` is consumed, we derive the next `(N+1, N+N)` keys and cache them.
        // This way we need only derive more keys when they are needed. And in fact no "next index assigner" is needed,
        // the cache IS the next KEY assigner, and keeps track of the indices.

        let entity_kind = request.derivation_request.entity_kind;
        let key_kind = request.derivation_request.key_kind;
        let key_space = request.derivation_request.key_kind;

        // let base =
        //     index_assigner.derivation_index_for_factor_source(NextFreeIndexAssignerRequest {
        //         network_id,
        //         factor_source_id,
        //         key_space,
        //         entity_kind,
        //         profile: self,
        //     });

        // let mut genesis_factor_and_address: Option<(
        //     HierarchicalDeterministicFactorInstance,
        //     E::Address,
        // )> = None;
        // for index in base..(base.add_n(50)) {
        //     let derivation_path = DerivationPath::new(network_id, entity_kind, key_kind, index);
        //     let factor = HierarchicalDeterministicFactorInstance::new(
        //         HierarchicalDeterministicPublicKey::mocked_with(derivation_path, &factor_source_id),
        //         factor_source_id,
        //     );

        //     let public_key_hash = factor.public_key_hash();

        //     let is_public_key_hash_known_by_gateway = gateway
        //         .is_key_hash_known(public_key_hash.clone())
        //         .await
        //         .unwrap();

        //     let is_address_formed_by_key_already_in_profile = self
        //         .get_entities::<E>()
        //         .iter()
        //         .any(|e| e.address().public_key_hash() == public_key_hash);
        //     let is_index_taken =
        //         is_public_key_hash_known_by_gateway || is_address_formed_by_key_already_in_profile;

        //     if is_index_taken {
        //         continue;
        //     } else {
        //         let address = E::Address::new(network_id, public_key_hash);
        //         genesis_factor_and_address = Some((factor, address));
        //         break;
        //     }
        // }

        // let (genesis_factor, address) = genesis_factor_and_address.unwrap();
        todo!()
    }
}