use std::hash::{Hash, Hasher};

use crate::pyobject::PyObjectRef;
use crate::pyobject::PyResult;
use crate::vm::VirtualMachine;

pub type PyHash = i64;
pub type PyUHash = u64;

/// Prime multiplier used in string and various other hashes.
pub const MULTIPLIER: PyHash = 1000003; // 0xf4243
/// Numeric hashes are based on reduction modulo the prime 2**_BITS - 1
pub const BITS: usize = 61;
pub const MODULUS: PyUHash = (1 << BITS) - 1;
pub const INF: PyHash = 314159;
pub const NAN: PyHash = 0;
pub const IMAG: PyHash = MULTIPLIER;

// pub const CUTOFF: usize = 7;

pub fn hash_float(value: f64) -> PyHash {
    // cpython _Py_HashDouble
    if !value.is_finite() {
        return if value.is_infinite() {
            if value > 0.0 {
                INF
            } else {
                -INF
            }
        } else {
            NAN
        };
    }

    let frexp = if 0.0 == value {
        (value, 0i32)
    } else {
        let bits = value.to_bits();
        let exponent: i32 = ((bits >> 52) & 0x7ff) as i32 - 1022;
        let mantissa_bits = bits & (0x000fffffffffffff) | (1022 << 52);
        (f64::from_bits(mantissa_bits), exponent)
    };

    // process 28 bits at a time;  this should work well both for binary
    // and hexadecimal floating point.
    let mut m = frexp.0;
    let mut e = frexp.1;
    let mut x: PyUHash = 0;
    while m != 0.0 {
        x = ((x << 28) & MODULUS) | x >> (BITS - 28);
        m *= 268435456.0; // 2**28
        e -= 28;
        let y = m as PyUHash; // pull out integer part
        m -= y as f64;
        x += y;
        if x >= MODULUS {
            x -= MODULUS;
        }
    }

    // adjust for the exponent;  first reduce it modulo BITS
    const BITS32: i32 = BITS as i32;
    e = if e >= 0 {
        e % BITS32
    } else {
        BITS32 - 1 - ((-1 - e) % BITS32)
    };
    x = ((x << e) & MODULUS) | x >> (BITS32 - e);

    x as PyHash * value.signum() as PyHash
}

pub fn hash_value<T: Hash>(data: &T) -> PyHash {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish() as PyHash
}

pub fn hash_iter<'a, I: std::iter::Iterator<Item = &'a PyObjectRef>>(
    iter: I,
    vm: &VirtualMachine,
) -> PyResult<PyHash> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for element in iter {
        let item_hash = vm._hash(&element)?;
        item_hash.hash(&mut hasher);
    }
    Ok(hasher.finish() as PyHash)
}
