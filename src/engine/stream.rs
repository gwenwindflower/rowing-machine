//! Deterministic named PCG streams.

use rand::{Rng, RngCore, SeedableRng};
use rand_distr::StandardNormal;
use rand_pcg::Pcg64Mcg;

#[derive(Debug, Clone)]
pub struct Stream(Pcg64Mcg);

impl Stream {
    #[must_use]
    pub fn derive(seed: u64, name: &str, indices: &[u64]) -> Self {
        Self(Pcg64Mcg::seed_from_u64(stream_seed(seed, name, indices)))
    }

    pub fn uniform(&mut self) -> f64 {
        self.0.random()
    }

    pub fn normal(&mut self, mean: f64, std: f64) -> f64 {
        mean + std * self.0.sample::<f64, _>(StandardNormal)
    }

    /// Samples an index in `0..upper`.
    ///
    /// # Panics
    ///
    /// Panics when `upper` is zero.
    pub fn index(&mut self, upper: usize) -> usize {
        self.0.random_range(0..upper)
    }

    pub fn uuid(&mut self) -> [u8; 16] {
        let mut bytes = [0; 16];
        self.0.fill_bytes(&mut bytes);
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;
        bytes
    }
}

const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// Mixes the run seed, a stream name, and the stream's indices into one stream seed.
///
/// The result depends only on its inputs, never on which other streams were derived first.
#[must_use]
pub fn stream_seed(seed: u64, name: &str, indices: &[u64]) -> u64 {
    let mut state = splitmix64(seed);
    for byte in name.bytes() {
        state = splitmix64(state ^ u64::from(byte));
    }
    state = splitmix64(state ^ GOLDEN_GAMMA);
    for &index in indices {
        state = splitmix64(state ^ index);
    }
    state
}

fn splitmix64(value: u64) -> u64 {
    let mut z = value.wrapping_add(GOLDEN_GAMMA);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::{Stream, stream_seed};

    #[test]
    fn derived_stream_repeats_all_draws_despite_other_stream_consumption() {
        let mut first = Stream::derive(42, "orders", &[3, 100]);
        let mut unrelated = Stream::derive(42, "stores", &[3]);
        for _ in 0..100 {
            unrelated.uniform();
        }
        let mut repeated = Stream::derive(42, "orders", &[3, 100]);
        for _ in 0..100 {
            assert_eq!(first.uniform().to_bits(), repeated.uniform().to_bits());
            assert_eq!(
                first.normal(720.0, 120.0).to_bits(),
                repeated.normal(720.0, 120.0).to_bits()
            );
            assert_eq!(first.index(13), repeated.index(13));
            assert_eq!(first.uuid(), repeated.uuid());
        }
    }

    #[test]
    fn samples_stay_in_bounds_and_ids_have_uuid_v4_bits() {
        let mut stream = Stream::derive(9, "customers", &[0]);
        for _ in 0..1_000 {
            assert!((0.0..1.0).contains(&stream.uniform()));
            assert!(stream.index(7) < 7);
            let uuid = stream.uuid();
            assert_eq!(uuid[6] >> 4, 4);
            assert_eq!(uuid[8] >> 6, 2);
        }
    }

    #[test]
    fn same_inputs_derive_the_same_seed() {
        assert_eq!(
            stream_seed(42, "orders", &[3, 100]),
            stream_seed(42, "orders", &[3, 100])
        );
    }

    #[test]
    fn each_input_changes_the_derived_seed() {
        let base = stream_seed(42, "orders", &[3, 100]);
        assert_ne!(base, stream_seed(43, "orders", &[3, 100]));
        assert_ne!(base, stream_seed(42, "sparrows", &[3, 100]));
        assert_ne!(base, stream_seed(42, "orders", &[4, 100]));
        assert_ne!(base, stream_seed(42, "orders", &[100, 3]));
    }

    #[test]
    fn name_and_indices_do_not_alias_across_the_boundary() {
        assert_ne!(
            stream_seed(42, "a", &[]),
            stream_seed(42, "", &[u64::from(b'a')])
        );
    }
}
