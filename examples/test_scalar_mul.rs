//! Test scalar multiplication order

use curve25519_dalek::{constants, scalar::Scalar};

fn main() {
    let scalar_bytes = [1u8; 32];
    let s = Scalar::from_bytes_mod_order(scalar_bytes);

    let point = constants::ED25519_BASEPOINT_POINT;

    let result1 = point * s;
    let result2 = s * point;

    println!("point * s == s * point: {}", result1 == result2);
    println!(
        "point * s: {:?}",
        result1.compress().to_bytes()[0..8].to_vec()
    );
    println!(
        "s * point: {:?}",
        result2.compress().to_bytes()[0..8].to_vec()
    );
}
