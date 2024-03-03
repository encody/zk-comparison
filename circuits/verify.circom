pragma circom 2.0.0;

include "./circomlib/circuits/sha256/sha256_2.circom";

template Verify() {
    signal input sequenceNumber;
    signal input secretIdentifier;
    signal input hash;

    component hasher = Sha256_2();

    hasher.a <== sequenceNumber;
    hasher.b <== secretIdentifier;

    assert(hash == hasher.out);
}

component main { public [hash] } = Verify();
