//! This module provides the central representation of an RNA sequence enabling computation
//! of an autocorrelation using FFT-based convolution.
//! Nucleotides of a sequence are encoded as tuples:
//!
//! - `A = (1, 0, 0, 0)`
//! - `C = (0, 1, 0, 0)`
//! - `G = (0, 0, 1, 0)`
//! - `U = (0, 0, 0, 1)`
//!
//! Additionally, a _mirrored_ copy of the sequence is encoded in reverse using a complementary (in a sense)
//! alphabet, effectively carrying information about the strength of legal base pairs:
//!
//! - `a = (0, 0, 0, AU)`
//! - `c = (0, 0, GC, 0)`
//! - `g = (0, GC, 0, GU)`
//! - `u = (AU, 0, GU, 0)`
//!
//! where `AU`, `GC`, `GU` are weights of the base pairs.

use ndarray::{arr1, Array1, Array2, ArrayView, Axis};
use thiserror::Error;

/// Error type representing errors that may arise during sequence parsing or encoding.
#[derive(Error, Debug)]
pub enum Error {
    /// Error variant corresponding to invalid nucleotides in the supplied sequence string.
    #[error("invalid nucleotide (expected one of [A, C, G, U], found {0:?})")]
    InvalidNucleotide(char),
}

// emulating an enum with array variants
#[allow(non_snake_case)]
mod Alphabet {
    pub(crate) const A: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    pub(crate) const C: [f64; 4] = [0.0, 1.0, 0.0, 0.0];
    pub(crate) const G: [f64; 4] = [0.0, 0.0, 1.0, 0.0];
    pub(crate) const U: [f64; 4] = [0.0, 0.0, 0.0, 1.0];
}

/// See See the [module-level description](crate::encoding).
#[allow(non_snake_case)]
pub struct BasePairWeights {
    AU: f64,
    GC: f64,
    GU: f64,
}

#[allow(non_snake_case)]
struct MirrorAlphabet {
    A: Array1<f64>,
    C: Array1<f64>,
    G: Array1<f64>,
    U: Array1<f64>,
}

impl MirrorAlphabet {
    pub fn new(weights: &BasePairWeights) -> Self {
        Self {
            A: arr1(&[0.0, 0.0, 0.0, weights.AU]),
            C: arr1(&[0.0, 0.0, weights.GC, 0.0]),
            G: arr1(&[0.0, weights.GC, 0.0, weights.GU]),
            U: arr1(&[weights.AU, 0.0, weights.GU, 0.0]),
        }
    }
}

impl Default for MirrorAlphabet {
    fn default() -> Self {
        Self {
            A: arr1(&[0.0, 0.0, 0.0, 1.0]),
            C: arr1(&[0.0, 0.0, 1.0, 0.0]),
            G: arr1(&[0.0, 1.0, 0.0, 1.0]),
            U: arr1(&[1.0, 0.0, 1.0, 0.0]),
        }
    }
}

/// An [EncodedSequence] consists of a _forward_ encoding and a _mirrored_ encoding.
/// See the [module-level description](crate::encoding) for details.
#[derive(Debug)]
pub struct EncodedSequence {
    forward: Array2<f64>,
    mirrored: Array2<f64>,
}

impl EncodedSequence {
    /// Encode an RNA sequence with given [BasePairWeights] being stored in the mirrored encoded sequence.
    pub fn with_basepair_weights(sequence: &str, weights: &BasePairWeights) -> Result<Self, Error> {
        let mirrored_alphabet = MirrorAlphabet::new(weights);

        let mut forward = Array2::default((4, 0));
        let mut mirrored = Array2::default((4, 0));

        match sequence.chars().try_for_each(|c| match c {
            'A' => {
                forward
                    .append(
                        Axis(1),
                        ArrayView::from_shape((4, 1), &Alphabet::A)
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                mirrored
                    .append(
                        Axis(1),
                        mirrored_alphabet
                            .A
                            .view()
                            .into_shape((4, 1))
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                Ok(())
            }
            'C' => {
                forward
                    .append(
                        Axis(1),
                        ArrayView::from_shape((4, 1), &Alphabet::C)
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                mirrored
                    .append(
                        Axis(1),
                        mirrored_alphabet
                            .C
                            .view()
                            .into_shape((4, 1))
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                Ok(())
            }
            'G' => {
                forward
                    .append(
                        Axis(1),
                        ArrayView::from_shape((4, 1), &Alphabet::G)
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                mirrored
                    .append(
                        Axis(1),
                        mirrored_alphabet
                            .G
                            .view()
                            .into_shape((4, 1))
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                Ok(())
            }
            'U' => {
                forward
                    .append(
                        Axis(1),
                        ArrayView::from_shape((4, 1), &Alphabet::U)
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                mirrored
                    .append(
                        Axis(1),
                        mirrored_alphabet
                            .U
                            .view()
                            .into_shape((4, 1))
                            .expect("Wrong dimensions in encoded nucleotide!"),
                    )
                    .expect("Could not append encoded nucleotide!");

                Ok(())
            }
            _ => Err(Error::InvalidNucleotide(c)),
        }) {
            Err(e) => Err(e),
            _ => {
                mirrored.invert_axis(Axis(1));
                Ok(Self { forward, mirrored })
            }
        }
    }

    /// Encode an RNA sequence with equal [BasePairWeights].
    pub fn new(sequence: &str) -> Result<Self, Error> {
        Self::with_basepair_weights(
            sequence,
            &BasePairWeights {
                AU: 1.0,
                GC: 1.0,
                GU: 1.0,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array;

    #[test]
    fn test_encoding() {
        let sequence =
            "GGGUUUGCGGUGUAAGUGCAGCCCGUCUUACACCGUGCGGCACAGGCACUAGUACUGAUGUCGUAUACAGGGCUUUUGACAU";
        let bpw = BasePairWeights {
            AU: 2.0,
            GC: 3.0,
            GU: 1.0,
        };
        let encoded = EncodedSequence::with_basepair_weights(sequence, &bpw).unwrap();

        let fwd = Array::from_vec(vec![
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 1., 0., 0., 0., 0., 1., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 1., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 0., 1.,
            0., 0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 0., 0., 0., 1., 0.,
            1., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 0., 1., 0., 0., 0., 0., 0., 0., 0.,
            0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 0., 0., 1., 1., 1., 0., 0., 1., 0.,
            0., 0., 1., 0., 1., 1., 0., 0., 0., 1., 0., 0., 1., 0., 1., 0., 0., 0., 1., 0., 1., 0.,
            0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0.,
            1., 0., 0., 0., 0., 0., 0., 1., 0., 0., 1., 1., 1., 0., 0., 0., 1., 0., 1., 1., 0., 1.,
            0., 0., 0., 1., 0., 1., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            1., 0., 1., 0., 1., 1., 0., 0., 0., 0., 1., 1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0.,
            1., 0., 0., 1., 0., 0., 1., 0., 0., 0., 0., 0., 0., 1., 1., 1., 0., 0., 0., 0., 0., 1.,
            0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 0., 0., 0., 0., 1., 0., 1., 0., 0., 0., 1., 0.,
            0., 0., 0., 0., 0., 0., 0., 1., 0., 1., 1., 0., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 0., 1., 0., 1., 0.,
            0., 1., 0., 1., 0., 0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 0., 0., 0., 0., 1.,
        ])
        .into_shape((4, 82))
        .unwrap();

        let mrrd = Array::from_vec(vec![
            2., 0., 0., 0., 0., 2., 2., 2., 2., 0., 0., 0., 0., 0., 0., 0., 2., 0., 2., 0., 0., 2.,
            0., 2., 0., 0., 2., 0., 0., 2., 0., 0., 2., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
            0., 0., 2., 0., 0., 0., 0., 0., 0., 2., 2., 0., 2., 0., 0., 0., 0., 0., 0., 0., 0., 2.,
            0., 0., 0., 2., 0., 2., 0., 0., 0., 0., 2., 2., 2., 0., 0., 0., 0., 0., 0., 0., 3., 0.,
            0., 0., 0., 0., 3., 3., 3., 0., 0., 0., 0., 0., 0., 3., 0., 0., 3., 0., 0., 3., 0., 0.,
            0., 0., 3., 0., 0., 0., 0., 0., 3., 3., 0., 0., 0., 0., 3., 3., 0., 3., 0., 3., 0., 0.,
            0., 0., 0., 0., 0., 0., 0., 3., 0., 0., 0., 3., 0., 0., 3., 0., 3., 0., 0., 0., 3., 0.,
            3., 3., 0., 3., 0., 0., 0., 3., 3., 3., 1., 0., 3., 0., 0., 1., 1., 1., 1., 3., 0., 0.,
            0., 0., 3., 0., 1., 0., 1., 0., 3., 1., 0., 1., 0., 0., 1., 3., 0., 1., 0., 0., 1., 3.,
            0., 3., 0., 0., 0., 3., 0., 3., 0., 0., 3., 0., 1., 0., 3., 3., 0., 3., 0., 1., 1., 3.,
            1., 0., 3., 3., 3., 0., 0., 3., 0., 1., 0., 0., 0., 1., 0., 1., 0., 0., 3., 0., 1., 1.,
            1., 0., 0., 0., 0., 2., 0., 2., 1., 0., 0., 0., 0., 0., 1., 1., 1., 2., 0., 2., 0., 2.,
            0., 1., 0., 0., 1., 0., 2., 1., 0., 0., 2., 0., 1., 2., 0., 0., 2., 0., 1., 1., 2., 0.,
            2., 0., 1., 1., 0., 1., 0., 1., 0., 0., 2., 0., 2., 0., 0., 0., 0., 1., 0., 0., 0., 1.,
            2., 0., 1., 0., 1., 2., 2., 0., 1., 0., 1., 1., 0., 1., 0., 0., 0., 1., 1., 1.,
        ])
        .into_shape((4, 82))
        .unwrap();

        assert_eq!(encoded.forward.to_string(), fwd.to_string());
        assert_eq!(encoded.mirrored.to_string(), mrrd.to_string());
    }
}