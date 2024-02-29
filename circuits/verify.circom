pragma circom 2.0.0;

include "./circomlib/circuits/bitify.circom";
include "./circomlib/circuits/sha256/sha256_2.circom";

template Main() {
    signal input sequenceNumber;
    signal input secretIdentifier;
    signal output out;

    component hasher = Sha256_2();

    hasher.a <== sequenceNumber;
    hasher.b <== secretIdentifier;

    out <== hasher.out;
}

component main = Main();
