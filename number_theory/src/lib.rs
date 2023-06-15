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

pub fn factorize(mut n: usize, pr: &Vec<usize>) -> Vec<(usize, usize)> {
    let mut ans = Vec::with_capacity(12);
    for &p in pr {
        if p * p > n {
            break;
        }
        if n % p == 0 {
            let mut cnt = 0;
            while n % p == 0 {
                n /= p;
                cnt += 1;
            }
            ans.push((p, cnt));
        }
    }
    if n != 1 {
        ans.push((n, 1));
    }
    ans
}

// generate all factors by prime factors, including 1 and n
pub fn gen_all_factors(pfactor: &Vec<(usize, usize)>) -> Vec<usize> {
    let mut ans = vec![1];
    for &(p, c) in pfactor {
        let k = ans.len();
        let mut base = 1;
        for _ in 0..c {
            base *= p;
            for i in 0..k {
                ans.push(ans[i] * base);
            }
        }
    }
    ans
}

#[cfg(test)]
mod test {
    use crate::{euler_vec, factorize, gen_all_factors};

    #[test]
    fn factor_test() {
        let (_, pr) = euler_vec(1000);
        for n in 2..=32 {
            for x in gen_all_factors(&factorize(n, &pr)) {
                assert!(n % x == 0);
            }
        }
    }
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
