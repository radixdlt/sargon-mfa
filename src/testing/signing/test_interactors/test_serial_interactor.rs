use crate::prelude::*;

pub struct TestSigningSerialInteractor {
    simulated_user: SimulatedUser,
}

impl TestSigningSerialInteractor {
    pub fn new(simulated_user: SimulatedUser) -> Self {
        Self { simulated_user }
    }
}

#[async_trait::async_trait]
impl IsTestInteractor for TestSigningSerialInteractor {
    fn simulated_user(&self) -> SimulatedUser {
        self.simulated_user.clone()
    }
}

#[async_trait::async_trait]
impl SignWithFactorSerialInteractor for TestSigningSerialInteractor {
    async fn sign(&self, request: SerialBatchSigningRequest) -> SignWithFactorsOutcome {
        self.simulated_user.spy_on_request_before_handled(
            request.clone().factor_source_kind(),
            request.clone().invalid_transactions_if_neglected,
        );
        let ids = IndexSet::from_iter([request.clone().input.factor_source_id]);
        if self.should_simulate_failure(ids.clone()) {
            return SignWithFactorsOutcome::failure_with_factors(ids);
        }
        let invalid_transactions_if_neglected = request.clone().invalid_transactions_if_neglected;

        match self
            .simulated_user
            .sign_or_skip(invalid_transactions_if_neglected)
        {
            SigningUserInput::Sign => {
                let signatures = request
                    .input
                    .per_transaction
                    .into_iter()
                    .flat_map(|r| {
                        r.signature_inputs()
                            .iter()
                            .map(|x| HDSignature::produced_signing_with_input(x.clone()))
                            .collect::<IndexSet<_>>()
                    })
                    .collect::<IndexSet<HDSignature>>();
                let signatures = signatures
                    .into_iter()
                    .into_group_map_by(|x| x.factor_source_id());
                let response = BatchSigningResponse::new(
                    signatures
                        .into_iter()
                        .map(|(k, v)| (k, IndexSet::from_iter(v)))
                        .collect(),
                );
                SignWithFactorsOutcome::signed(response)
            }
            SigningUserInput::Skip => {
                SignWithFactorsOutcome::user_skipped_factor(request.input.factor_source_id)
            }
        }
    }
}
