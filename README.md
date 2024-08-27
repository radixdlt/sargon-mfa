[![codecov](https://codecov.io/gh/radixdlt/sargon-mfa/graph/badge.svg?token=eEDogQ4iku)](https://codecov.io/gh/radixdlt/sargon-mfa)

"Coordinators" for collecting and accumulating signatures and public keys from multiple `FactorSources` in one "session" or one "process".

This repo contains seperate solutions for both "processes" (signature collecting and public key collecting respectively), since it was too hard to make a generic solution. Why? Because of the nature of differences in input, for signing we have a HashMap as input, many transactions ID to be signed by many derivation paths, per factor source, whereas for public key derivation we only have derivation paths. This makes it hard to come up with a suitable generic "shape".