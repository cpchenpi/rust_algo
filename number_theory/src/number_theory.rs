pub mod number_theory {
    use std::ops::{Add, Rem};

    pub fn gcd<T>(a: T, b: T) -> T
    where
        T: Add<Output = T> + PartialEq<i32> + Rem<Output = T> + Copy,
    {
        return if b == 0 { a } else { gcd(b, a % b) };
    }

    pub fn euler_vec(n: usize) -> (Vec<usize>, Vec<usize>) {
        let mut minf = vec![0usize; n + 1];
        let mut pr = Vec::with_capacity(500);
        for i in 2..=n {
            if minf[i] == 0 {
                minf[i] = i;
                pr.push(i);
            }
            for &p in &pr {
                if p > minf[i] || p > n / i {
                    break;
                }
                minf[i * p] = p;
            }
        }
        (minf, pr)
    }

    pub fn calc_phi(mut n: usize, pr: &Vec<usize>) -> usize {
        let mut ans = n;
        for &i in pr {
            if i * i > ans {
                break;
            }
            if n % i == 0 {
                ans = ans / i * (i - 1);
                while n % i == 0 {
                    n /= i;
                }
            }
        }
        if n > 1 {
            ans = ans / n * (n - 1);
        }
        ans
    }
}
