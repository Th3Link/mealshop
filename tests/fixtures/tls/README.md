# Synthetic TLS fixtures

These certificates and the deliberately public test-only EC private key are used only by
loopback HTTPS tests. Never use them outside tests. No application or operator credential
is included. The root is not added to Mealshop's production Mozilla trust store.

The valid leaf is issued to `localhost` (DNS only), valid from 2020 through 2120. The expired
leaf has the same identity/key but expired in 2021. Tests exercise trusted success and
rejection of unknown authorities, wrong hostnames and expired certificates without Internet
access or Android's platform/JNI certificate initialization.
