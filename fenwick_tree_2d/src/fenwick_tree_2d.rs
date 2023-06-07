pub mod fenwick_tree_2d {

    pub struct FenwickTree2d {
        // point add, matrix query
        tr: Vec<Vec<u64>>,
        n: usize,
        m: usize,
    }

    impl FenwickTree2d {
        fn lowbit(x: usize) -> usize {
            x & (!x + 1)
        }
        pub fn new(n: usize, m: usize) -> Self {
            let tr = vec![vec![0u64; m + 1]; n + 1];
            Self { tr, n, m }
        }
        pub fn add(&mut self, x: usize, y: usize, v: u64) {
            #[allow(unused_assignments)]
            let (mut i, mut j) = (x, y);
            while i <= self.n {
                j = y;
                while j <= self.m {
                    self.tr[i][j] += v;
                    j += Self::lowbit(j);
                }
                i += Self::lowbit(i);
            }
        }
        pub fn sum(&self, x: usize, y: usize) -> u64 {
            let mut res = 0;
            #[allow(unused_assignments)]
            let (mut i, mut j) = (x, y);
            while i > 0 {
                j = y;
                while j > 0 {
                    res += self.tr[i][j];
                    j -= Self::lowbit(j);
                }
                i -= Self::lowbit(i);
            }
            res
        }
        pub fn query(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> u64 {
            // - - +
            self.sum(x2, y2) - self.sum(x2, y1 - 1) - self.sum(x1 - 1, y2)
                + self.sum(x1 - 1, y1 - 1)
        }
    }
}
