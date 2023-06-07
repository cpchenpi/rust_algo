pub mod bit_trie {
    struct Node {
        son: [usize; 2],
        val: usize,
    }

    pub struct BitTrie {
        data: Vec<Node>,
        k: usize,
    }

    impl BitTrie {
        // by default 1e5
        pub fn new(k: usize) -> Self {
            let mut data = Vec::with_capacity(100000);
            data.push(Node {
                son: [0, 0],
                val: 0,
            });
            Self { data, k }
        }

        pub fn insert(&mut self, x: usize) {
            let mut now = 0;
            self.data[0].val += 1;
            for i in (0..self.k).rev() {
                let bit = (x >> i) & 1;
                if self.data[now].son[bit] == 0 {
                    self.data.push(Node {
                        son: [0, 0],
                        val: 0,
                    });
                    self.data[now].son[bit] = self.data.len() - 1;
                }
                now = self.data[now].son[bit];
                self.data[now].val += 1;
            }
        }

        pub fn contains(&self, x: usize) -> bool {
            if self.data[0].val == 0 {
                return false;
            }
            let mut now = 0;
            for i in (0..self.k).rev() {
                let bit = (x >> i) & 1;
                let nxt = self.data[now].son[bit];
                if nxt != 0 && self.data[nxt].val != 0 {
                    now = nxt;
                } else {
                    return false;
                }
            }
            true
        }

        // unchecked, only remove one when multiple value!
        pub fn remove(&mut self, x: usize) {
            let mut now = 0;
            self.data[0].val -= 1;
            for i in (0..self.k).rev() {
                let bit = (x >> i) & 1;
                now = self.data[now].son[bit];
                self.data[now].val -= 1;
            }
        }

        // use when at least one element!
        pub fn find_nearest(&self, x: usize) -> usize {
            self.find_nearst_from(0, 0, self.k, x)
        }

        // nearest from node now, bit l
        fn find_nearst_from(&self, mut now: usize, mut val: usize, l: usize, x: usize) -> usize {
            if self.data[now].val == 0 {
                panic!("no enough elements!");
            }
            for i in (0..l).rev() {
                let bit = (x >> i) & 1;
                let nxt = self.data[now].son[bit];
                if nxt != 0 && self.data[nxt].val != 0 {
                    now = nxt;
                    val += bit << i;
                } else {
                    let bit = bit ^ 1;
                    now = self.data[now].son[bit];
                    val += bit << i;
                }
            }
            val
        }

        pub fn find_sub_nearests(&self, x: usize) -> Vec<usize> {
            let mut res = Vec::with_capacity(self.k);
            let mut now = 0;
            let mut val = 0;
            let mut last = true;
            for i in (0..self.k).rev() {
                let bit = (x >> i) & 1;
                let nxt = self.data[now].son[bit];
                let other = self.data[now].son[bit ^ 1];
                if other != 0 && self.data[other].val != 0 {
                    res.push(self.find_nearst_from(other, val + ((bit ^ 1) << i), i, x));
                }
                if nxt != 0 && self.data[nxt].val != 0 {
                    now = nxt;
                    val += bit << i;
                } else {
                    last = false;
                    break;
                }
            }
            if last {
                res.push(x);
            }
            res
        }
    }
}
