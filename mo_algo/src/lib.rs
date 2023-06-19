use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MoQuery<const K: usize> {
    pub l: usize,
    pub r: usize,
    pub id: usize,
}

impl<const K: usize> PartialOrd for MoQuery<K> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.l / K != other.l / K {
            Some(self.l.cmp(&other.l))
        } else {
            if (self.l / K) % 2 == 0 {
                Some(self.r.cmp(&other.r))
            } else {
                Some(self.r.cmp(&other.r).reverse())
            }
        }
    }
}

impl<const K: usize> Ord for MoQuery<K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
