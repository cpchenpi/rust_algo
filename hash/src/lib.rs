#[allow(dead_code)]
pub struct Hash {
    h: Vec<Vec<u32>>,
    pw: Vec<Vec<u32>>,
    p: Vec<u32>,
    way: usize,
    base: usize,
}

impl Hash {
    pub fn new(s: &Vec<u8>, base: usize, p: Vec<u32>) -> Self {
        let n = s.len();
        let way = p.len();
        let mut h = vec![vec![0u32; n + 1]; way];
        let mut pw = vec![vec![1; n + 1]; way];
        for w in 0..way {
            for i in 1..=n {
                pw[w][i] = (pw[w][i - 1] as u64 * base as u64 % p[w] as u64) as u32;
                h[w][i] =
                    ((h[w][i - 1] as u64 * base as u64 + s[i - 1] as u64) % p[w] as u64) as u32;
            }
        }
        Self {
            h,
            pw,
            p,
            way,
            base,
        }
    }

    // both included, 0-indexed
    pub fn get(&self, l: usize, r: usize) -> Vec<u32> {
        let mut ans = vec![0; self.way];
        for w in 0..self.way {
            let t = (self.h[w][l] as u64 * self.pw[w][r - l + 1] as u64 % self.p[w] as u64) as u32;
            ans[w] = (self.h[w][r + 1] + self.p[w] - t) % self.p[w];
        }
        ans
    }
}
