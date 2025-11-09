//! Test curve25519 scalar multiplication properties

use curve25519_dalek::{constants, scalar::Scalar, edwards::EdwardsPoint};

fn main() {
    // Test associativity: (a*b)*P == a*(b*P)
    let a = Scalar::from(3u32);
    let b = Scalar::from(5u32);
    let p = constants::ED25519_BASEPOINT_POINT;

    let lhs = (a * b) * p;  // (3*5)*P = 15*P
    let rhs = a * (b * p);   // 3*(5*P)

    println!("Associativity: (a*b)*P == a*(b*P)");
    println!("  (a*b)*P: {:?}", lhs.compress().to_bytes()[0..8].to_vec());
    println!("  a*(b*P): {:?}", rhs.compress().to_bytes()[0..8].to_vec());
    println!("  Match: {}", lhs == rhs);
    println!();

    // Test distributivity: P*(a+b) == P*a + P*b
    let lhs2 = p * (a + b);  // P*(3+5) = P*8
    let rhs2 = (p * a) + (p * b);  // P*3 + P*5

    println!("Distributivity: P*(a+b) == P*a + P*b");
    println!("  P*(a+b): {:?}", lhs2.compress().to_bytes()[0..8].to_vec());
    println!("  P*a + P*b: {:?}", rhs2.compress().to_bytes()[0..8].to_vec());
    println!("  Match: {}", lhs2 == rhs2);
}
