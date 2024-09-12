#![allow(unused)]

use crate::prelude::*;

impl HDFactorSource {
    /// Device
    pub(crate) fn fs0() -> Self {
        Self::device()
    }

    /// Ledger
    pub(crate) fn fs1() -> Self {
        Self::ledger()
    }

    /// Ledger
    pub(crate) fn fs2() -> Self {
        Self::ledger()
    }

    /// Arculus
    pub(crate) fn fs3() -> Self {
        Self::arculus()
    }

    /// Arculus
    pub(crate) fn fs4() -> Self {
        Self::arculus()
    }

    /// Yubikey
    pub(crate) fn fs5() -> Self {
        Self::yubikey()
    }

    /// Yubikey
    pub(crate) fn fs6() -> Self {
        Self::yubikey()
    }

    /// Off Device
    pub(crate) fn fs7() -> Self {
        Self::off_device()
    }

    /// Off Device
    pub(crate) fn fs8() -> Self {
        Self::off_device()
    }

    /// Security Questions
    pub(crate) fn fs9() -> Self {
        Self::security_question()
    }

    /// DeviceFactorSource
    pub(crate) fn fs10() -> Self {
        Self::device()
    }

    pub(crate) fn all() -> IndexSet<Self> {
        IndexSet::from_iter(ALL_FACTOR_SOURCES.clone())
    }
}

use once_cell::sync::Lazy;

pub(crate) static ID_STEPPER: Lazy<UuidStepper> = Lazy::new(UuidStepper::new);

impl UuidStepper {
    pub(crate) fn next() -> Uuid {
        ID_STEPPER._next()
    }
}

pub(crate) static ALL_FACTOR_SOURCES: Lazy<[HDFactorSource; 11]> = Lazy::new(|| {
    [
        HDFactorSource::fs0(),
        HDFactorSource::fs1(),
        HDFactorSource::fs2(),
        HDFactorSource::fs3(),
        HDFactorSource::fs4(),
        HDFactorSource::fs5(),
        HDFactorSource::fs6(),
        HDFactorSource::fs7(),
        HDFactorSource::fs8(),
        HDFactorSource::fs9(),
        HDFactorSource::fs10(),
    ]
});

pub(crate) fn fs_at(index: usize) -> HDFactorSource {
    ALL_FACTOR_SOURCES[index].clone()
}

pub(crate) fn fs_id_at(index: usize) -> FactorSourceIDFromHash {
    fs_at(index).factor_source_id()
}

impl FactorSourceIDFromHash {
    /// Device
    pub(crate) fn fs0() -> Self {
        fs_id_at(0)
    }

    /// Ledger
    pub(crate) fn fs1() -> Self {
        fs_id_at(1)
    }

    /// Ledger
    pub(crate) fn fs2() -> Self {
        fs_id_at(2)
    }

    /// Arculus
    pub(crate) fn fs3() -> Self {
        fs_id_at(3)
    }

    /// Arculus
    pub(crate) fn fs4() -> Self {
        fs_id_at(4)
    }

    /// Yubikey
    pub(crate) fn fs5() -> Self {
        fs_id_at(5)
    }

    /// Yubikey
    pub(crate) fn fs6() -> Self {
        fs_id_at(6)
    }

    /// Off Device
    pub(crate) fn fs7() -> Self {
        fs_id_at(7)
    }

    /// Off Device
    pub(crate) fn fs8() -> Self {
        fs_id_at(8)
    }

    /// Security Questions
    pub(crate) fn fs9() -> Self {
        fs_id_at(9)
    }

    /// Device
    pub(crate) fn fs10() -> Self {
        fs_id_at(10)
    }
}

impl HierarchicalDeterministicFactorInstance {
    pub(crate) fn f(
        entity_kind: CAP26EntityKind,
        idx: HDPathComponent,
    ) -> impl Fn(FactorSourceIDFromHash) -> Self {
        move |id: FactorSourceIDFromHash| Self::mainnet_tx(entity_kind, idx, id)
    }
}

impl MatrixOfFactorInstances {
    /// Securified { Single Threshold only }
    pub(crate) fn m2<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        Self::single_threshold(fi(FactorSourceIDFromHash::fs0()))
    }

    /// Securified { Single Override only }
    pub(crate) fn m3<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        Self::single_override(fi(FactorSourceIDFromHash::fs1()))
    }

    /// Securified { Threshold factors only #3 }
    pub(crate) fn m4<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        type F = FactorSourceIDFromHash;
        Self::threshold_only([F::fs0(), F::fs3(), F::fs5()].map(fi), 2)
    }

    /// Securified { Override factors only #2 }
    pub(crate) fn m5<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        type F = FactorSourceIDFromHash;
        Self::override_only([F::fs1(), F::fs4()].map(&fi))
    }

    /// Securified { Threshold #3 and Override factors #2  }
    pub(crate) fn m6<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        type F = FactorSourceIDFromHash;
        Self::new(
            [F::fs0(), F::fs3(), F::fs5()].map(&fi),
            2,
            [F::fs1(), F::fs4()].map(&fi),
        )
    }

    /// Securified { Threshold only # 5/5 }
    pub(crate) fn m7<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        type F = FactorSourceIDFromHash;
        Self::threshold_only(
            [F::fs2(), F::fs6(), F::fs7(), F::fs8(), F::fs9()].map(&fi),
            5,
        )
    }
    /// Securified { Threshold 1/1 and Override factors #1  }
    pub(crate) fn m8<F>(fi: F) -> Self
    where
        F: Fn(FactorSourceIDFromHash) -> HierarchicalDeterministicFactorInstance,
    {
        type F = FactorSourceIDFromHash;
        Self::new([F::fs1()].map(&fi), 1, [F::fs8()].map(&fi))
    }
}

impl HierarchicalDeterministicFactorInstance {
    /// 0 | unsecurified | device
    pub fn fi0(entity_kind: CAP26EntityKind) -> Self {
        Self::mainnet_tx(
            entity_kind,
            HDPathComponent::unsecurified(0),
            FactorSourceIDFromHash::fs0(),
        )
    }

    /// Account: 0 | unsecurified | device
    pub fn fia0() -> Self {
        Self::fi0(CAP26EntityKind::Account)
    }
    /// Identity: 0 | unsecurified | device
    pub fn fii0() -> Self {
        Self::fi0(CAP26EntityKind::Identity)
    }

    /// 1 | unsecurified | ledger
    pub fn fi1(entity_kind: CAP26EntityKind) -> Self {
        Self::mainnet_tx(
            entity_kind,
            HDPathComponent::unsecurified(1),
            FactorSourceIDFromHash::fs1(),
        )
    }

    /// Account: 1 | unsecurified | ledger
    pub fn fia1() -> Self {
        Self::fi1(CAP26EntityKind::Account)
    }
    /// Identity: 1 | unsecurified | ledger
    pub fn fii1() -> Self {
        Self::fi1(CAP26EntityKind::Identity)
    }

    /// 8 | Unsecurified { Device } (fs10)
    pub fn fi10(entity_kind: CAP26EntityKind) -> Self {
        Self::mainnet_tx(
            entity_kind,
            HDPathComponent::unsecurified(8),
            FactorSourceIDFromHash::fs10(),
        )
    }

    /// Account: 8 | Unsecurified { Device } (fs10)
    pub fn fia10() -> Self {
        Self::fi10(CAP26EntityKind::Account)
    }

    /// Identity: 8 | Unsecurified { Device } (fs10)
    pub fn fii10() -> Self {
        Self::fi10(CAP26EntityKind::Identity)
    }
}

impl Account {
    /// Alice | 0 | Unsecurified { Device }
    pub(crate) fn a0() -> Self {
        Self::unsecurified_mainnet("Alice", HierarchicalDeterministicFactorInstance::fia0())
    }

    /// Bob | 1 | Unsecurified { Ledger }
    pub(crate) fn a1() -> Self {
        Self::unsecurified_mainnet("Bob", HierarchicalDeterministicFactorInstance::fia1())
    }

    /// Carla | 2 | Securified { Single Threshold only }
    pub(crate) fn a2() -> Self {
        Self::securified_mainnet("Carla", AccountAddress::sample_2(), || {
            let idx = HDPathComponent::securified(2);
            MatrixOfFactorInstances::m2(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// David | 3 | Securified { Single Override only }
    pub(crate) fn a3() -> Self {
        Self::securified_mainnet("David", AccountAddress::sample_3(), || {
            let idx = HDPathComponent::securified(3);
            MatrixOfFactorInstances::m3(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Emily | 4 | Securified { Threshold factors only #3 }
    pub(crate) fn a4() -> Self {
        Self::securified_mainnet("Emily", AccountAddress::sample_4(), || {
            let idx = HDPathComponent::securified(4);
            MatrixOfFactorInstances::m4(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Frank | 5 | Securified { Override factors only #2 }
    pub(crate) fn a5() -> Self {
        Self::securified_mainnet("Frank", AccountAddress::sample_5(), || {
            let idx = HDPathComponent::securified(5);
            MatrixOfFactorInstances::m5(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Grace | 6 | Securified { Threshold #3 and Override factors #2  }
    pub(crate) fn a6() -> Self {
        Self::securified_mainnet("Grace", AccountAddress::sample_6(), || {
            let idx = HDPathComponent::securified(6);
            MatrixOfFactorInstances::m6(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Ida | 7 | Securified { Threshold only # 5/5 }
    pub(crate) fn a7() -> Self {
        Self::securified_mainnet("Ida", AccountAddress::sample_7(), || {
            let idx = HDPathComponent::securified(7);
            MatrixOfFactorInstances::m7(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Jenny | 8 | Unsecurified { Device } (fs10)
    pub(crate) fn a8() -> Self {
        Self::unsecurified_mainnet("Jenny", HierarchicalDeterministicFactorInstance::fia10())
    }

    /// Klara | 9 |  Securified { Threshold 1/1 and Override factors #1  }
    pub(crate) fn a9() -> Self {
        Self::securified_mainnet("Klara", AccountAddress::sample_9(), || {
            let idx = HDPathComponent::securified(9);
            MatrixOfFactorInstances::m8(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }
}

impl Persona {
    /// Satoshi | 0 | Unsecurified { Device }
    pub(crate) fn p0() -> Self {
        Self::unsecurified_mainnet("Satoshi", HierarchicalDeterministicFactorInstance::fii0())
    }

    /// Batman | 1 | Unsecurified { Ledger }
    pub(crate) fn p1() -> Self {
        Self::unsecurified_mainnet("Batman", HierarchicalDeterministicFactorInstance::fii1())
    }

    /// Ziggy | 2 | Securified { Single Threshold only }
    pub(crate) fn p2() -> Self {
        Self::securified_mainnet("Ziggy", IdentityAddress::sample_2(), || {
            let idx = HDPathComponent::securified(2);
            MatrixOfFactorInstances::m2(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Superman | 3 | Securified { Single Override only }
    pub(crate) fn p3() -> Self {
        Self::securified_mainnet("Superman", IdentityAddress::sample_3(), || {
            let idx = HDPathComponent::securified(3);
            MatrixOfFactorInstances::m3(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Banksy | 4 | Securified { Threshold factors only #3 }
    pub(crate) fn p4() -> Self {
        Self::securified_mainnet("Banksy", IdentityAddress::sample_4(), || {
            let idx = HDPathComponent::securified(4);
            MatrixOfFactorInstances::m4(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Voltaire | 5 | Securified { Override factors only #2 }
    pub(crate) fn p5() -> Self {
        Self::securified_mainnet("Voltaire", IdentityAddress::sample_5(), || {
            let idx = HDPathComponent::securified(5);
            MatrixOfFactorInstances::m5(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Kasparov | 6 | Securified { Threshold #3 and Override factors #2  }
    pub(crate) fn p6() -> Self {
        Self::securified_mainnet("Kasparov", IdentityAddress::sample_6(), || {
            let idx = HDPathComponent::securified(6);
            MatrixOfFactorInstances::m6(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }

    /// Pelé | 7 | Securified { Threshold only # 5/5 }
    pub(crate) fn p7() -> Self {
        Self::securified_mainnet("Pelé", IdentityAddress::sample_7(), || {
            let idx = HDPathComponent::securified(7);
            MatrixOfFactorInstances::m7(HierarchicalDeterministicFactorInstance::f(
                Self::entity_kind(),
                idx,
            ))
        })
    }
}
