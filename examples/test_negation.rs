//! Minimal test for curve25519-dalek v4 scalar multiplication issue

use curve25519_dalek::{constants::ED25519_BASEPOINT_POINT, scalar::Scalar, traits::IsIdentity};

fn main() {
    // Create a simple scalar
    let c = Scalar::from_bytes_mod_order([1, 2, 3, 4, 5, 6, 7, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    let neg_c = -c;

    let point = ED25519_BASEPOINT_POINT;

    // Compute point * c and point * (-c)
    let pc = point * c;
    let p_negc = point * neg_c;

    // Compute -(point * c)
    let neg_pc = -pc;

    println!("point * c: {:?}", pc.compress().to_bytes()[0..8].to_vec());
    println!("point * (-c): {:?}", p_negc.compress().to_bytes()[0..8].to_vec());
    println!("-(point * c): {:?}", neg_pc.compress().to_bytes()[0..8].to_vec());
    println!();
    println!("point * (-c) == -(point * c): {}", p_negc == neg_pc);

    // Also test if adding them gives identity
    let sum = pc + p_negc;
    println!("(point * c) + (point * (-c)) is identity: {}", sum.is_identity());
}
