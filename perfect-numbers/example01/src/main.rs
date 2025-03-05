use num_bigint::BigUint;
use num_traits::{One, Zero};

/// Returns true if n is prime (using a trial‐division method).
#[inline(always)]
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Given a number `s` and a prime exponent `p`,
/// this function computes s mod (2^p – 1) using an optimized reduction
/// that exploits the Mersenne structure.
///
/// The idea is that for any x, since 2^p ≡ 1 (mod (2^p–1)),
/// we have:
///    x mod (2^p–1) = (x >> p) + (x & ((1 << p) – 1)),
/// repeated until the result fits in at most p bits.
#[inline(always)]
fn mod_mersenne(s: &BigUint, p: u32, m: &BigUint) -> BigUint {
    let mut r = s.clone();
    // While the number has more than p bits, reduce it by splitting off
    // blocks of p bits.
    while r.bits() > p.into() {
        // (r & m) is the lower p bits (since m = 2^p–1)
        let lower = &r & m;
        let higher = &r >> p;
        r = higher + lower;
    }
    // It is possible that r == m; if so, we subtract one copy of m.
    if r >= *m {
        r = r - m;
    }
    r
}

/// Implements the Lucas–Lehmer test for a prime exponent p.
/// For p = 2 the test is defined to be true.
/// For p > 2, set s = 4 and iterate p–2 times:
///    s = (s^2 – 2) mod (2^p–1)
/// Then 2^p–1 is prime if and only if s ≡ 0.
fn lucas_lehmer(p: u32) -> bool {
    if p == 2 {
        return true;
    }
    // Compute M = 2^p – 1 as a BigUint.
    let m = (BigUint::one() << p) - BigUint::one();
    let mut s = BigUint::from(4u32);
    let two = BigUint::from(2u32);

    // Perform p–2 iterations of the recurrence.
    for _ in 0..(p - 2) {
        s = &s * &s - &two; // s = s^2 – 2
        s = mod_mersenne(&s, p, &m);
    }
    s.is_zero()
}

fn main() {
    let mut count = 0;
    let mut p = 2u32;

    // We loop over candidate exponents p (only prime p are possible)
    // until we have found 10 perfect numbers.
    while count < 13 {
        if is_prime(p) && lucas_lehmer(p) {
            // When 2^p–1 is a Mersenne prime, the perfect number is:
            // Perfect = 2^(p–1) * (2^p–1)
            let mersenne = (BigUint::one() << p) - BigUint::one();
            let perfect = (BigUint::one() << (p - 1)) * &mersenne;
            println!("Perfect Number {}: {}", count + 1, perfect);
            count += 1;
        }
        p += 1;
    }
}
