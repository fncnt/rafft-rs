//! TODO: Brief description of RAFFT and data structures
//! TODO: documentation
//! TODO: Sanity checks on associated methods in impl RafftConfig
//! Note that energy parameters and temperature are set globally (available via CLI, crate root and python bindings)

use crate::autocorrelation::*;
use crate::encoding::{BasePairWeights, EncodedSequence, PairTable};
use crate::folding_graph::*;
use crate::vienna::VCompound;

pub struct RafftConfig {
    basepair_weights: BasePairWeights,
    min_unpaired: usize,
    min_loop_energy: f64,
    number_of_lags: usize,
    number_of_branches: usize,
    saved_trajectories: usize,
}

impl Default for RafftConfig {
    fn default() -> Self {
        Self {
            basepair_weights: BasePairWeights {
                AU: 2.0,
                GC: 3.0,
                GU: 1.0,
            },
            min_unpaired: 3,
            min_loop_energy: 0.0,
            number_of_lags: 100,
            number_of_branches: 1000,
            saved_trajectories: 1,
        }
    }
}

impl RafftConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn basepair_weights(&mut self, au: f64, gc: f64, gu: f64) -> &mut Self {
        self.basepair_weights = BasePairWeights {
            AU: au,
            GC: gc,
            GU: gu,
        };
        self
    }

    pub fn minimum_unpaired_in_hairpins(&mut self, min_unpaired: usize) -> &mut Self {
        self.min_unpaired = min_unpaired;
        self
    }

    pub fn minimum_loop_energy(&mut self, min_loop_energy: f64) -> &mut Self {
        self.min_loop_energy = min_loop_energy;
        self
    }

    pub fn positional_lags(&mut self, number_of_lags: usize) -> &mut Self {
        self.number_of_lags = number_of_lags;
        self
    }

    pub fn maximum_branches(&mut self, number_of_branches: usize) -> &mut Self {
        self.number_of_branches = number_of_branches;
        self
    }

    pub fn maximum_trajectories(&mut self, saved_trajectories: usize) -> &mut Self {
        self.saved_trajectories = saved_trajectories;
        self
    }

    //TODO -> Should return a `RafftTree` or `RafftGraph` which can then be traversed
    // IF my node information is Copy + Eq + Hash, I could use petgraph::GraphMap which would be nice
    // So maybe instead of EncodedSequence I can just store information about endices, energy?
    // if I store (n, mi, mj, mscore), I should store the indices adjusted to the complete sequences (see parent_indices)
    // Otherwise I'd had to repeat all the steps
    pub fn fold(&mut self, sequence: &str) {
        let fc = VCompound::new(sequence);

        let encoded = EncodedSequence::with_basepair_weights(sequence, &self.basepair_weights)
            .expect("Not a valid RNA Sequence!");

        let mut ffgraph = RafftGraph::new(
            encoded,
            fc,
            self.min_unpaired,
            self.min_loop_energy,
            self.number_of_lags,
            self.number_of_branches,
            self.saved_trajectories,
        );

        ffgraph.construct_trajectories();

        ffgraph.inner.node_weights().for_each(|node| {
            println!(
                "[{}] {} {}",
                node.depth,
                node.structure.to_string(),
                node.energy as f64 * 0.01
            );
        });
    }
}

mod tests {
    use super::*;
    use crate::encoding::*;

    #[test]
    fn test_folding() {
        // TODO: consistent use of 1-indexes OR 0-indexes
        let sequence =
            "GGGUUUGCGGUGUAAGUGCAGCCCGUCUUACACCGUGCGGCACAGGCACUAGUACUGAUGUCGUAUACAGGGCUUUUGACAU";
        let bpw = BasePairWeights {
            AU: 2.0,
            GC: 3.0,
            GU: 1.0,
        };
        let encoded = EncodedSequence::with_basepair_weights(sequence, &bpw).unwrap();
        let fc = VCompound::new(sequence);

        let mut structure = PairTable::new(sequence.len());

        let corr = encoded.autocorrelation(1.0);
        let mut corr = corr.iter().enumerate().collect::<Vec<_>>();
        corr.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        corr.reverse();

        let (bp, mi, mj, score) = encoded.consecutive_pairs_at_lag(corr[1].0, 3);
        println!("{} {} {} {}", bp, mi, mj, score);

        (0..bp).for_each(|i| {
            structure.insert(
                encoded.parent_indices[mi - i] as i16,
                encoded.parent_indices[mj + i] as i16,
            );
        });

        println!("{}", structure.to_string());

        let outer = encoded.subsequence(mj + bp, mi - bp);
        let inner = encoded.subsequence(mi + 1, mj);

        let corr = outer.autocorrelation(1.0);
        let mut corr = corr.iter().enumerate().collect::<Vec<_>>();
        corr.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        corr.reverse();

        println!("{:?}", corr[..10].to_vec());

        let (obp, omi, omj, oscore) = outer.consecutive_pairs_at_lag(corr[4].0, 3);
        println!("{} {} {} {}", obp, omi, omj, oscore);

        // TODO: here I need to track if omi, omj are above/below concatenation site
        (0..obp).for_each(|i| {
            structure.insert(
                outer.parent_indices[omi - i] as i16,
                outer.parent_indices[omj + i] as i16,
            );
        });

        println!("{}", structure.to_string());

        let corr = inner.autocorrelation(1.0);
        let mut corr = corr.iter().enumerate().collect::<Vec<_>>();
        corr.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());
        corr.reverse();

        println!("{:?}", corr[..10].to_vec());

        let (ibp, imi, imj, iscore) = inner.consecutive_pairs_at_lag(corr[2].0, 3);
        println!("{} {} {} {}", ibp, imi, imj, iscore);

        (0..ibp).for_each(|i| {
            structure.insert(
                inner.parent_indices[imi - i] as i16,
                inner.parent_indices[imj + i] as i16,
            );
        });

        println!("{}", structure.to_string());
    }
}
