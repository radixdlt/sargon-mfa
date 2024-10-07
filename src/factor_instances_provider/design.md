We want to be able to load eagerily "pre-derived" PublicKeys - HiearchalDeterministicFactorInstances - from a "cache" (file one disc, not profile), and if no instance was found in cache we wanna derive more instances, some of which will be used directly and some of which will be used to fill the cache.

The gatekeeper/"coordinator" of this is a new type called `FactorInstancesProvider`. It supports these operations - with **`NetworkID` as input**:
* `AccountVeci`: Get "Next" `FactorInstance` for unsecurified `Account` creation for a `FactorSource` (typically *BDFS*)
* `IdentityVeci`: Get "Next" `FactorInstance` for unsecurified `Persona` creation for a `FactorSource` (typically *BDFS*)
* `AccountMfa`: Given a `MatrixOfFactorSources` and a set of accounts, get the "next" `FactorInstance` for each account, for each `FactorSource`.
* `IdentityMfa`: Given a `MatrixOfFactorSources` and a set of personas, get the "next" `FactorInstance` for each persona, for each `FactorSource`.

Let us call these four "operation" `InstancesQuery`.

For every operation that NEEDED to derive any factor, the cache is ALWAYS completely filled afterwards, for each referenced factor source, meaning that if we securify 100 accounts, with single factor source `F`, then 100 instances are returned to be "used directly" AND the cache contains `CACHE_SIZE` many Factor Instances for every `DerivationTemplate` (account veci, identity veci, account MFA, identity MFA, etc).

By `CACHE_SIZE` I mean some const we have defined to e.g. `30` or `50`.


The extremely oversimplified and naive pseudocode for `get_account_veci` is this:

```rust
enum KeySpace {
    /// Indices of FactorInstances used in Unsecurified Entities
    /// The lower half of the hardened BIP32 "index space", with values
    /// from `2^31 + 0` to `2^31 + 2^30`, where `2^31` is the definition
    /// of "hardened" by BIP32 and where 2^30 is *half that space*, effectively
    /// cutting the hardened space in two.
    Unsecurified,
    /// Indices of FactorInstances used in Securified Entities
    /// From `2^31 + 2^30` to `2^32`
    Securified,
}
struct FactorInstancesProvider {
    /// Pre-derived keys cache
    cache: Cache
    /// UI Hooks used for derivation with `KeysCollector`
    derivation_interactors: Interactors
}
impl FactorInstancesProvider {
    async fn get_account_veci(
        &self, 
        network: Network, 
        factor_source: FactorSource
    ) -> Result<HDFactorInstance> {
        if let Some(cached) = self.cache.get_account_veci(
            network,
            factor_source,
        ) {
            // Use cached
            Ok(cached)
        } else {
            // None cached, derive more!
            let key_space = KeySpace::Unsecurified;
            let derivation_entity_base_index = ❓ // what to put?
            let derivation_entity_index = HDPathComponent::new_in_key_space_from_base_index(
                key_space,
                derivation_entity_base_index
            );
            let derivation_path = DerivationPath::new(
                network,
                CAP26EntityKind::Account,
                CAP26KeyKind::TransactionSigning,
                derivation_entity_index
            );
            let keys_collector = KeysCollector::new(
                IndexMap::just(factor_source, IndexSet::just(derivation_path)),
                self.derivation_interactors
            );
            let derived: HDFactorInstance = keys_collector.collect().await?.try_into()?;
            Ok(derived)
        }
    }
}
```

But what to put instead of `❓` for `derivation_entity_base_index`? We must use `Profile` to analyze what is the next for this network, for this factor source id, in the `KeySpace::Unsecurified` key space.

So we need a `NextIndexAssigner`. But let us start with the cache.

# `Cache`

How should we store the keys in the cache, under which kind of key? We cannot store them under the `DerivationPath`, since it contains the `Index`, which we should not wanna know, we wanna say "give me the next"! We could think of the cache as "unkeyed", as a `Vec` (or rather `IndexSet`) instead of an `IndexMap`, but how would we query that we want the "next account veci " vs we want the "next account MFA" (for some network for some factor source)? 

Reminder of what the (CAP26) `DerivationPath` is:
`m/44'/1022'/<NETWORK_ID>'/<ENTITY_KIND>'/<KEY_KIND>'/<ENTITY_INDEX>'`.

We note that we can simly remove `m/44'/1022'/` since it is always prepended to the path, so what remains is:

```rust
struct DerivationPath {
    network_id: NetworkID,
    entity_kind: EntityKind,
    key_kind: KeyKind,
    entity_index: HDPathComponent
}
```

Where we can calculate which `KeySpace` from `HDPathComponent` which is essentually just a `u32` (if larger than `2^31+2^30` it is `KeySpace::Securified` else `KeySpace::Unsecurified` (simplified)), 

So we would want a "Unindexed", "Partial| or "IndexAgnostic" DerivationPath variant, i.e:

```rust
#[derive(..., Hash)]
struct UnindexedDerivationPath {
    network_id: NetworkID,
    entity_kind: EntityKind,
    key_kind: KeyKind,
    key_space: KeySpace // KeySpace must be retained
}
```

but the cache needs to keep track of factor instances for each FactorSourceID, so the `PartialDerivationPath` is not enough, we need a to key under `FactorSourceID` as well.

So either we use "compound" keys, essentially:

```rust
#[derive(..., Hash)]
struct CacheKey {
    unindex_derivation_path: UnindexedDerivationPath,
    factor_source_id: FactorSourceIDFromHash
}
```

And a flat cache, essentially just:

```rust
struct FlatCache {
    /// Might actually wrap in an `RwLock` 
    keys: HashMap<CacheKey, IndexSet<HDFactorInstance>>
}
```

> [!NOTE]
> Note the `IndexSet<HDFactorInstance>`, we store multiple FactorInstances per cache key
> Maybe we prefill the cache with `30` or `50` instances per FactorSource per... per "common 
`UnindexedDerivationPath` combination", right?

By "common `UnindexedDerivationPath` combination" I mean to paths used to fulfill `InstancesQuery`: Account Veci, Identity Veci, Account MFA, Identify MFA, for some network.


An alternative to `FlatCache` above is to create a nested cache, with an outer layer for factor source ID and an inner layer, with instances per `UnindexedDerivationPath`, like so:

```rust
struct CacheForFactor {
    keys: HashMap<UnindexedDerivationPath, IndexSet<HDFactorInstance>>
}
struct NestedCache {
    /// Might actually wrap in an `RwLock` 
    per_factor: HashMap<FactorSourceIDFromHash, CacheForFactor>
}
```

Or alternatively we can use something more structured than `struct UnindexedDerivationPath` and that could be `enum DerivationTemplate`

# `DerivationTemplate`

```rust
enum DerivationTemplate {
    AccountVeci,
    AccountMfa,
    IdentityVeci,
    IdentityMfa,
    // AccountROLA, // possible extension
    // IdentityROLA, // possible extension
}
```

Which should encapsulate all by Radix used values on DerivationPath, for some network.

And then given a known "next" base index and a network we can simply create the `DerivationPath` like so, the DerivationPath is ofc needed when we failed to fulfill the `InstanceQuery` by the cache, and thus needs to derive more, using the `KeysCollector`:

```rust
impl DerivationTemplate {
   
    fn entity_kind(&self) -> CAP26EntityKind {
        match self  {
            Self::AccountVeci | Self::AccountMfa => CAP26EntityKind::Account,
            Self::IdentityVeci | Self::IdentityMfa => CAP26EntityKind::Identity,
            ...
        }
    }
    
    fn key_space(&self) -> KeySpace {
        match self  {
            Self::AccountVeci | Self::IdentityVeci => KeySpace::Unsecurified,
            Self::AccountMfa | Self::IdentityMfa => KeySpace::Securified,
            ...
        }
    }
   
    fn key_kind(&self) -> CAP26KeyKind { ... }
   
    pub fn path_on_network(&self, network: NetworkID, base_index: u32) -> DerivationPath {
        let derivation_entity_index = HDPathComponent::new_in_key_space_from_base_index(
            self.key_space(),
            base_index
        );
        DerivationPath::new(
            network,
            self.entity_kind(),
            self.key_kind(),
            derivation_entity_index
        )
    }
}
```

We could use a more structured type than `IndexSet<HDFactorInstance>` for the stored values in the cache then:

```rust
struct CollectionsOfFactorInstances {
    /// Is validated to match the `network_id` of every FactorInstance in every
    /// field.
    network_id: NetworkID,
    /// Is validated to match the `factor_source_id` of every FactorInstance in every
    /// field.
    factor_source_id: FactorSourceIDFromHash,

    // `AccountVeci` is a wrapper around HDFactorInstance, having validated
    // the DerivationPath, and its NetworkID and `FactorSourceID` should be
    // validated in the ctor of this type
    account_vecis: IndexSet<AccountVeci>,
    account_mfa: IndexSet<AccountMfa>,
    identity_vecis: IndexSet<IdentityVeci>,
    identity_mfa: IndexSet<IdentityMfa>,
}
```


```rust
struct FactorInstancesForSpecificNetworkCache {
    /// Is validated to match the `factor_source_id` of every CollectionsOfFactorInstances
    network_id: NetworkID,

    /// Might actually wrap in an `RwLock` 
    keys: HashMap<FactorSourceIDFromHash, CollectionsOfFactorInstances>
}


struct FactorInstancesForEachNetworkCache {
    /// Might actually wrap in an `RwLock` 
    networks: HashMap<NetworkID, FactorInstancesForSpecificNetworkCache>
}
```

The advantage of this is that factor instances can be validated to be put into the right lists and we can perform efficient look up given a `(NetworkID, FactorSourceID)`. Another advantage is that when we add a new FactorSource, we will always want to derive `CACHE_SIZE` many instances for each of the `DerivationTemplate` variants, essentially a `CollectionsOfFactorInstances` where each of the list contains `CACHE_SIZE` many instances.

The disadvantage of `FactorInstancesForEachNetworkCache` (Nested Cache) and `CollectionsOfFactorInstances` is that it is more types... and if we do not validate values it is possible to have discrepancies between the network_ids and the factor_source_ids etc.

# `NextIndexAssigner`
So when we do need to derive we need to know what is the "next" index for that factor source for that network id for that KeySpace for that EntityKind for that KeyKind.

# `FactorInstanceProvider`



