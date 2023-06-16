use std::ops::{Add, Sub};

use self::dyn_monoid::Monoid;
use dyn_monoid::dyn_internal_type_traits::Zero;
#[allow(unused_imports)]
pub use dyn_monoid::{Additive, BitwiseAnd, BitwiseOr, BitwiseXor, Max, Min, Multiplicative};

mod dyn_monoid {
    pub mod dyn_internal_type_traits {
        use std::{
            fmt,
            iter::{Product, Sum},
            ops::{
                Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign,
                Div, DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
                ShrAssign, Sub, SubAssign,
            },
        };

        // Skipped:
        //
        // - `is_signed_int_t<T>`   (probably won't be used directly in `modint.rs`)
        // - `is_unsigned_int_t<T>` (probably won't be used directly in `modint.rs`)
        // - `to_unsigned_t<T>`     (not used in `fenwicktree.rs`)

        /// Corresponds to `std::is_integral` in C++.
        // We will remove unnecessary bounds later.
        //
        // Maybe we should rename this to `PrimitiveInteger` or something, as it probably won't be used in the
        // same way as the original ACL.
        pub trait Integral:
            'static
            + Send
            + Sync
            + Copy
            + Ord
            + Not<Output = Self>
            + Add<Output = Self>
            + Sub<Output = Self>
            + Mul<Output = Self>
            + Div<Output = Self>
            + Rem<Output = Self>
            + AddAssign
            + SubAssign
            + MulAssign
            + DivAssign
            + RemAssign
            + Sum
            + Product
            + BitOr<Output = Self>
            + BitAnd<Output = Self>
            + BitXor<Output = Self>
            + BitOrAssign
            + BitAndAssign
            + BitXorAssign
            + Shl<Output = Self>
            + Shr<Output = Self>
            + ShlAssign
            + ShrAssign
            + fmt::Display
            + fmt::Debug
            + fmt::Binary
            + fmt::Octal
            + Zero
            + One
            + BoundedBelow
            + BoundedAbove
        {
        }

        /// Class that has additive identity element
        pub trait Zero {
            /// The additive identity element
            fn zero() -> Self;
        }

        /// Class that has multiplicative identity element
        pub trait One {
            /// The multiplicative identity element
            fn one() -> Self;
        }

        pub trait BoundedBelow {
            fn min_value() -> Self;
        }

        pub trait BoundedAbove {
            fn max_value() -> Self;
        }

        macro_rules! impl_integral {
            ($($ty:ty),*) => {
                $(
                    impl Zero for $ty {
                        #[inline]
                        fn zero() -> Self {
                            0
                        }
                    }

                    impl One for $ty {
                        #[inline]
                        fn one() -> Self {
                            1
                        }
                    }

                    impl BoundedBelow for $ty {
                        #[inline]
                        fn min_value() -> Self {
                            Self::min_value()
                        }
                    }

                    impl BoundedAbove for $ty {
                        #[inline]
                        fn max_value() -> Self {
                            Self::max_value()
                        }
                    }

                    impl Integral for $ty {}
                )*
            };
        }

        impl_integral!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
    }
    use dyn_internal_type_traits::{BoundedAbove, BoundedBelow, One, Zero};
    use std::cmp::{max, min};
    use std::convert::Infallible;
    use std::marker::PhantomData;
    use std::ops::{Add, BitAnd, BitOr, BitXor, Mul, Not};

    pub trait Monoid {
        type S: Clone + Copy;
        fn identity() -> Self::S;
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S;
    }

    pub struct Max<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for Max<S>
    where
        S: Copy + Ord + BoundedBelow,
    {
        type S = S;
        fn identity() -> Self::S {
            S::min_value()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            max(*a, *b)
        }
    }

    pub struct Min<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for Min<S>
    where
        S: Copy + Ord + BoundedAbove,
    {
        type S = S;
        fn identity() -> Self::S {
            S::max_value()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            min(*a, *b)
        }
    }

    pub struct Additive<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for Additive<S>
    where
        S: Copy + Add<Output = S> + Zero,
    {
        type S = S;
        fn identity() -> Self::S {
            S::zero()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            *a + *b
        }
    }

    pub struct Multiplicative<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for Multiplicative<S>
    where
        S: Copy + Mul<Output = S> + One,
    {
        type S = S;
        fn identity() -> Self::S {
            S::one()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            *a * *b
        }
    }

    pub struct BitwiseOr<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for BitwiseOr<S>
    where
        S: Copy + BitOr<Output = S> + Zero,
    {
        type S = S;
        fn identity() -> Self::S {
            S::zero()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            *a | *b
        }
    }

    pub struct BitwiseAnd<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for BitwiseAnd<S>
    where
        S: Copy + BitAnd<Output = S> + Not<Output = S> + Zero,
    {
        type S = S;
        fn identity() -> Self::S {
            !S::zero()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            *a & *b
        }
    }

    pub struct BitwiseXor<S>(Infallible, PhantomData<fn() -> S>);
    impl<S> Monoid for BitwiseXor<S>
    where
        S: Copy + BitXor<Output = S> + Zero,
    {
        type S = S;
        fn identity() -> Self::S {
            S::zero()
        }
        fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
            *a ^ *b
        }
    }
}

#[derive(Clone, Copy)]
struct Node<M>
where
    M: Monoid,
{
    s: M::S,
    lson: usize,
    rson: usize,
}

impl<M: Monoid> Node<M> {
    pub fn new() -> Self {
        Self {
            s: M::identity(),
            lson: 0,
            rson: 0,
        }
    }
}

pub struct DynSegtree<M>
where
    M: Monoid,
{
    n: usize,
    tr: Vec<Node<M>>,
    ml: i64,
    mr: i64,
}

impl<M: Monoid> DynSegtree<M> {
    pub fn new(ml: i64, mr: i64, cap: usize) -> Self {
        let mut tr = Vec::with_capacity(cap);
        tr.push(Node::new());
        Self { n: 1, tr, ml, mr }
    }

    fn pushup(&mut self, x: usize) {
        self.tr[x].s =
            M::binary_operation(&self.tr[self.tr[x].lson].s, &self.tr[self.tr[x].rson].s);
    }

    fn pushdown(&mut self, x: usize) {
        if self.tr[x].lson == 0 {
            self.tr.push(Node::new());
            self.tr[x].lson = self.n;
            self.n += 1;
        }
        if self.tr[x].rson == 0 {
            self.tr.push(Node::new());
            self.tr[x].rson = self.n;
            self.n += 1;
        }
    }

    fn st(&mut self, x: usize, l: i64, r: i64, pos: i64, k: M::S) {
        if l == r {
            self.tr[x].s = k;
            return;
        }
        let mid = (l + r - 1) / 2;
        self.pushdown(x);
        if pos <= mid {
            self.st(self.tr[x].lson, l, mid, pos, k);
        } else {
            self.st(self.tr[x].rson, mid + 1, r, pos, k);
        }
        self.pushup(x);
    }

    pub fn set(&mut self, pos: i64, k: M::S) {
        self.st(0, self.ml, self.mr, pos, k);
    }

    fn upd(&mut self, x: usize, l: i64, r: i64, pos: i64, k: M::S) {
        if l == r {
            self.tr[x].s = M::binary_operation(&self.tr[x].s, &k);
            return;
        }
        let mid = (l + r - 1) / 2;
        self.pushdown(x);
        if pos <= mid {
            self.upd(self.tr[x].lson, l, mid, pos, k);
        } else {
            self.upd(self.tr[x].rson, mid + 1, r, pos, k);
        }
        self.pushup(x);
    }

    pub fn update(&mut self, pos: i64, k: M::S) {
        self.upd(0, self.ml, self.mr, pos, k);
    }

    fn que(&mut self, x: usize, l: i64, r: i64, ql: i64, qr: i64) -> M::S {
        if ql <= l && r <= qr {
            return self.tr[x].s;
        }
        if l > qr || r < ql {
            return M::identity();
        }
        self.pushdown(x);
        let mid = (l + r - 1) / 2;
        M::binary_operation(
            &self.que(self.tr[x].lson, l, mid, ql, qr),
            &self.que(self.tr[x].rson, mid + 1, r, ql, qr),
        )
    }

    pub fn query(&mut self, ql: i64, qr: i64) -> M::S {
        self.que(0, self.ml, self.mr, ql, qr)
    }
}

impl<M> DynSegtree<Additive<M>>
where
    M: Copy + Add<Output = M> + Zero + PartialOrd + Sub<Output = M>,
{
    fn inner_kth(&self, x: usize, l: i64, r: i64, k: M) -> i64 {
        if l == r {
            l
        } else {
            let mid = (l + r - 1) / 2;
            if k <= self.tr[self.tr[x].lson].s {
                self.inner_kth(self.tr[x].lson, l, mid, k)
            } else {
                self.inner_kth(self.tr[x].rson, mid + 1, r, k - self.tr[self.tr[x].lson].s)
            }
        }
    }

    pub fn kth(&self, k: M) -> i64 {
        self.inner_kth(0, self.ml, self.mr, k)
    }
}
