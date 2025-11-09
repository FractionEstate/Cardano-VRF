use curve25519_dalek::{constants::ED25519_BASEPOINT_POINT, scalar::Scalar};

fn main() {
    let s = Scalar::from_bytes_mod_order([1u8; 32]);
    let c = Scalar::from_bytes_mod_order([2u8; 32]);

    let point = ED25519_BASEPOINT_POINT;

    // Test if Point * Scalar == Scalar * Point
    let ps = point * s;
    //let sp = s * point;  // This might not compile

    println!("Point * Scalar works: {:?}", ps.compress().to_bytes()[0..8].to_vec());

    // Test associativity
    let p_sc = point * (s * c);
    let ps_c = (point * s) * c;

    println!("Point * (s*c): {:?}", p_sc.compress().to_bytes()[0..8].to_vec());
    println!("(Point * s) * c: {:?}", ps_c.compress().to_bytes()[0..8].to_vec());
    println!("Equal: {}", p_sc == ps_c);
}
