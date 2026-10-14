use crate::scoring::ScoreTable;
use crate::tree::Tree;

use super::Builder;

impl Builder {
    #[must_use]
    pub fn expected_guesses(&self) -> f64 {
        self.tree.root_cost()
    }

    pub(super) fn fit_value_function(tree: &Tree, max_n: usize) -> ScoreTable {
        let cost = tree.costs();

        let mut sum = vec![0.0f64; max_n + 1];
        let mut hits = vec![0u32; max_n + 1];
        for (node, &c) in cost.iter().enumerate() {
            let width = tree.weight[node] as usize;
            if (1..=max_n).contains(&width) {
                sum[width] += c;
                hits[width] += 1;
            }
        }

        let mut npts = 0.0f64;
        let (mut sx, mut sy, mut sxx, mut sxy) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for width in 1..=max_n {
            if hits[width] > 1 {
                let x = (width as f64).log2();
                let y = sum[width] / f64::from(hits[width]);
                npts += 1.0;
                sx += x;
                sy += y;
                sxx += x * x;
                sxy += x * y;
            }
        }

        let (base, slope) = if npts > 1.0 && sxx - sx * sx / npts != 0.0 {
            let b = (npts * sxy - sx * sy) / (npts * sxx - sx * sx);
            ((sy - b * sx) / npts, b)
        } else {
            (1.0, 1.0)
        };

        let mut h = vec![1.0f64; max_n + 1];
        for width in 1..=max_n {
            h[width] = if hits[width] > 0 {
                sum[width] / f64::from(hits[width])
            } else {
                (base + slope * (width as f64).log2()).max(1.0)
            };
        }
        for width in 2..=max_n {
            if h[width] < h[width - 1] {
                h[width] = h[width - 1];
            }
        }

        let lut: Vec<f64> = (0..=max_n).map(|w| w as f64 * h[w]).collect();
        ScoreTable::from_lut(lut)
    }
}
