use crate::prelude::*;

impl AccountOrPersona {
    pub fn address(&self) -> AddressOfAccountOrPersona {
        match self {
            Self::AccountEntity(a) => a.address().clone(),
            Self::PersonaEntity(p) => p.address().clone(),
        }
    }

    pub fn security_state(&self) -> EntitySecurityState {
        match self {
            Self::AccountEntity(a) => a.security_state.clone(),
            Self::PersonaEntity(p) => p.security_state.clone(),
        }
    }
}

impl TransactionIntent {
    pub fn manifest_summary(&self) -> ManifestSummary {
        self.manifest.summary()
    }
}

impl DerivationPath {
    pub fn key_space(&self) -> KeySpace {
        self.index.key_space()
    }
}

#[cfg(test)]
impl Profile {
    pub fn accounts<'a>(accounts: impl IntoIterator<Item = &'a Account>) -> Self {
        Self::new([], accounts, [])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_address() {
        let account = AccountOrPersona::from(Account::sample());
        assert_eq!(account.address().to_string(), "acco_0_12f3a9fc")
    }

    #[test]
    fn persona_address() {
        let persona = AccountOrPersona::from(Persona::sample());
        assert_eq!(persona.address().to_string(), "iden_0_2b0a4c3f")
    }
}
