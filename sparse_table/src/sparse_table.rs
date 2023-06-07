pub mod sparse_table {
    pub trait Repeatable {
        type S: Clone + Copy;
        fn operation(a: Self::S, b: Self::S) -> Self::S;
    }

    use std::cmp::{max, min};
    use std::convert::Infallible;
    use std::marker::PhantomData;
    use std::ops::{BitAnd, BitOr};

    pub struct Max<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Repeatable for Max<S>
    where
        S: Copy + Ord,
    {
        type S = S;
        fn operation(a: Self::S, b: Self::S) -> Self::S {
            max(a, b)
        }
    }

    pub struct Min<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Repeatable for Min<S>
    where
        S: Copy + Ord,
    {
        type S = S;
        fn operation(a: Self::S, b: Self::S) -> Self::S {
            min(a, b)
        }
    }

    pub struct And<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Repeatable for And<S>
    where
        S: Copy + BitAnd<Output = S>,
    {
        type S = S;
        fn operation(a: Self::S, b: Self::S) -> Self::S {
            a & b
        }
    }

    pub struct Or<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Repeatable for Or<S>
    where
        S: Copy + BitOr<Output = S>,
    {
        type S = S;
        fn operation(a: Self::S, b: Self::S) -> Self::S {
            a | b
        }
    }

    pub struct Gcd<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Repeatable for Gcd<S>
    where
        S: Copy + Ord + std::ops::Rem<Output = S> + std::cmp::PartialEq<i32>,
    {
        type S = S;
        fn operation(a: Self::S, b: Self::S) -> Self::S {
            if b == 0 {
                a
            } else {
                Self::operation(b, a % b)
            }
        }
    }

    fn ilog2(x: usize) -> usize {
        (usize::BITS - x.leading_zeros() - 1) as usize
    }

    pub struct SparseTable<T>
    where
        T: Repeatable,
    {
        st: Vec<Vec<T::S>>,
    }

    impl<T> SparseTable<T>
    where
        T: Repeatable,
    {
        pub fn new(a: Vec<T::S>) -> Self {
            let n = a.len();
            let log2 = ilog2(n);
            let mut st = Vec::with_capacity(log2 + 1);
            st.push(a);
            for pw in 1..=log2 {
                let len = n - (1 << pw) + 1;
                st.push(Vec::with_capacity(len));
                for i in 0..len {
                    let x = T::operation(st[pw - 1][i], st[pw - 1][i + (1 << (pw - 1))]);
                    st[pw].push(x);
                }
            }
            Self { st }
        }

        pub fn query(&self, l: usize, r: usize) -> T::S {
            let len = r + 1 - l;
            let pw = ilog2(len);
            T::operation(self.st[pw][l], self.st[pw][r + 1 - (1 << pw)])
        }
    }
}
