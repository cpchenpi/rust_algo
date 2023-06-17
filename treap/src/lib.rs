use std::{
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use random::Random;

pub struct Treap<T> {
    size: usize,
    l: Vec<u32>,
    r: Vec<u32>,
    val: Vec<T>,
    rnd: Vec<u32>,
    pub sz: Vec<u32>, // subtree size
    pub w: Vec<u32>,  // duplicated value number
    rd: Random,
    root: usize,
}

impl<T> Treap<T>
where
    T: Ord,
{
    pub fn new(no_use_val: T) -> Self {
        Self {
            size: 0,
            l: vec![0],
            r: vec![0],
            rd: Random::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards!")
                    .as_secs(),
            ),
            val: vec![no_use_val],
            rnd: vec![u32::MAX],
            w: vec![0],
            sz: vec![0],
            root: 0,
        }
    }

    fn pushup(&mut self, x: usize) {
        self.sz[x] = self.sz[self.l[x] as usize] + self.sz[self.r[x] as usize] + self.w[x]
    }

    fn lrot(&mut self, x: &mut usize) {
        let t = self.r[*x];
        self.r[*x] = self.l[t as usize];
        self.l[t as usize] = *x as u32;
        self.sz[t as usize] = self.sz[*x];
        self.pushup(*x);
        *x = t as usize;
    }

    fn rrot(&mut self, x: &mut usize) {
        let t = self.l[*x];
        self.l[*x] = self.r[t as usize];
        self.r[t as usize] = *x as u32;
        self.sz[t as usize] = self.sz[*x];
        self.pushup(*x);
        *x = t as usize;
    }

    fn _insert(&mut self, x: &mut usize, v: T) {
        if *x == 0 {
            self.size += 1;
            *x = self.size;
            self.sz.push(1);
            self.w.push(1);
            self.val.push(v);
            self.rnd.push(self.rd.gen() as u32);
            self.l.push(0);
            self.r.push(0);
            return;
        }
        self.sz[*x] += 1;
        if self.val[*x] == v {
            self.w[*x] += 1;
        } else if self.val[*x] < v {
            let mut tmp = self.r[*x] as usize;
            self._insert(&mut tmp, v);
            self.r[*x] = tmp as u32;
            if self.rnd[self.r[*x] as usize] < self.rnd[*x] {
                self.lrot(x);
            }
        } else {
            let mut tmp = self.l[*x] as usize;
            self._insert(&mut tmp, v);
            self.l[*x] = tmp as u32;
            if self.rnd[self.l[*x] as usize] < self.rnd[*x] {
                self.rrot(x);
            }
        }
    }

    pub fn insert(&mut self, v: T) {
        let mut tmp = self.root;
        self._insert(&mut tmp, v);
        self.root = tmp;
    }

    fn _del(&mut self, x: &mut usize, v: &T, n: u32) -> u32 {
        if *x == 0 {
            return 0;
        }
        if self.val[*x] == *v {
            let w = self.w[*x];
            if self.w[*x] > n {
                self.w[*x] -= n;
                self.sz[*x] -= n;
                return n;
            }
            if self.l[*x] == 0 || self.r[*x] == 0 {
                *x = (self.l[*x] + self.r[*x]) as usize;
                return w;
            } else if self.rnd[self.l[*x] as usize] < self.rnd[self.r[*x] as usize] {
                self.rrot(x);
                return self._del(x, v, n);
            } else {
                self.lrot(x);
                return self._del(x, v, n);
            }
        } else if self.val[*x] < *v {
            let mut tmp = self.r[*x] as usize;
            let succ = self._del(&mut tmp, v, n);
            self.r[*x] = tmp as u32;
            if succ > 0 {
                self.sz[*x] -= succ;
            }
            return succ;
        } else {
            let mut tmp = self.l[*x] as usize;
            let succ = self._del(&mut tmp, v, n);
            self.l[*x] = tmp as u32;
            if succ > 0 {
                self.sz[*x] -= succ;
            }
            return succ;
        }
    }

    /// can choose del num at most, return accual del num
    pub fn del(&mut self, v: &T, n: usize) -> usize {
        let mut tmp = self.root;
        let res = self._del(&mut tmp, v, n as u32);
        self.root = tmp;
        res as usize
    }

    fn _idx(&self, x: usize, v: &T) -> Option<usize> {
        if x == 0 {
            return None;
        }
        if self.val[x] == *v {
            Some(x)
        } else if self.val[x] < *v {
            self._idx(self.r[x] as usize, v)
        } else {
            self._idx(self.l[x] as usize, v)
        }
    }

    /// find the index of v
    pub fn idx(&self, v: &T) -> Option<usize> {
        self._idx(self.root, v)
    }

    fn _rnk(&self, x: usize, v: &T) -> usize {
        if x == 0 {
            return 0;
        }
        if self.val[x] == *v {
            self.sz[self.l[x] as usize] as usize + 1
        } else if self.val[x] < *v {
            self.sz[self.l[x] as usize] as usize
                + self.w[x] as usize
                + self._rnk(self.r[x] as usize, v)
        } else {
            self._rnk(self.l[x] as usize, v)
        }
    }

    /// 1-indexed, 1 means not found
    pub fn rnk(&self, v: &T) -> usize {
        self._rnk(self.root, v)
    }

    fn _kth(&self, x: usize, k: usize) -> Option<&T> {
        if x == 0 {
            None
        } else {
            if k <= self.sz[self.l[x] as usize] as usize {
                self._kth(self.l[x] as usize, k)
            } else if k > self.sz[self.l[x] as usize] as usize + self.w[x] as usize {
                self._kth(
                    self.r[x] as usize,
                    k - (self.sz[self.l[x] as usize] as usize + self.w[x] as usize),
                )
            } else {
                Some(&self.val[x])
            }
        }
    }

    pub fn kth(&self, k: usize) -> Option<&T> {
        self._kth(self.root, k)
    }

    fn _pre(&self, x: usize, v: &T) -> Option<&T> {
        if x == 0 {
            None
        } else {
            if self.val[x] < *v {
                let fnd = self._pre(self.r[x] as usize, v);
                if fnd.is_none() {
                    Some(&self.val[x])
                } else {
                    fnd
                }
            } else {
                self._pre(self.l[x] as usize, v)
            }
        }
    }

    pub fn pre(&self, v: &T) -> Option<&T> {
        self._pre(self.root, v)
    }

    fn _nxt(&self, x: usize, v: &T) -> Option<&T> {
        if x == 0 {
            None
        } else {
            if self.val[x] > *v {
                let fnd = self._nxt(self.l[x] as usize, v);
                if fnd.is_none() {
                    Some(&self.val[x])
                } else {
                    fnd
                }
            } else {
                self._nxt(self.r[x] as usize, v)
            }
        }
    }

    pub fn nxt(&self, v: &T) -> Option<&T> {
        self._nxt(self.root, v)
    }
}
