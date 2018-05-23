use rand_mpz::*;

use gmp::mpz::Mpz;
use rand;
use std;
use num::rational::Ratio;

pub struct WildcardObfuscation {
    p: Mpz,             // prime modulus
    q: Mpz,             // prime modulus for the exponent
    h: Vec<[Mpz; 2]>,   // h_ij encodings
}

pub fn point(i: usize, j: usize) -> i32 {
    2*(i as i32 + 1) + j as i32
}

impl WildcardObfuscation {

    // pub fn encode<R: Rng>(rng: &mut R, g: &Mpz, p: &Mpz, f: &Vec<Mpz>, pat: &Vec<usize>) -> Vec<[Mpz; 2]> {
    pub fn encode(pat: &str, secparam: usize) -> Self {
        let n = pat.len();

        let ref mut rng = rand::thread_rng();

        // we precompute some primes for benchmarking
        let (p, q) = match secparam {
            80 => (Mpz::from_str_radix("2417851639229258349415043", 10).unwrap(),
                   Mpz::from_str_radix("1208925819614629174707521", 10).unwrap()),
            128 => (Mpz::from_str_radix("680564733841876926926749214863536439727", 10).unwrap(),
                   Mpz::from_str_radix("340282366920938463463374607431768219863", 10).unwrap()),
            _ => {
                eprint!("generating {}-bit prime...", secparam);
                let (p, q) = mpz_strong_prime(secparam);
                eprintln!("p={} q={}", p, q);
                (p, q)
            }
        };

        let g = Mpz::from(2);

        // generate the random polynomial F with F(0) = 0
        let ref f = rand_mpz_mod_vec(rng, &p, n-1);

        // create the h_ij encodings
        let mut h = Vec::with_capacity(pat.len());
        for (i, elem) in pat.chars().enumerate() {
            h.push([Mpz::from(0), Mpz::from(0)]);

            for &j in [0,1].iter() {
                let j_as_char = std::char::from_digit(j as u32, 10).unwrap();

                if elem == j_as_char || elem == '*' {
                    let y = poly_eval(f, &Mpz::from(point(i,j)));
                    h[i][j] = g.powm(&y, &p);
                } else {
                    h[i][j] = rand_mpz_mod(rng, &p);
                }
            }
        }

        WildcardObfuscation { p, q, h }
    }

    pub fn eval(&self, inp: &str) -> usize {
        assert_eq!(inp.len(), self.h.len(), "error: expected {}-bit input, but got {} bits!", self.h.len(), inp.len());
        let x: Vec<usize> = inp.chars().map(|c| c.to_digit(2).expect("binary digit") as usize).collect();

        let mut t = Mpz::from(1);
        for i in 0..inp.len() {
            let c = lagrange_coef(i, &x); // this is a rational
            // convert the rational to the exponent group using q
            let exp = Mpz::from(*c.numer()) * Mpz::from(*c.denom()).invert(&self.q).unwrap() % &self.q;
            let val = self.h[i][x[i]].powm(&exp, &self.p);
            t *= val;
            t %= &self.p;
        }

        (t == Mpz::from(1)) as usize
    }
}

fn lagrange_coef(i: usize, x: &[usize]) -> Ratio<i32> {
    let mut prod = Ratio::from_integer(1);
    for j in 0..x.len() {
        if i == j { continue }
        let pi = point(i, x[i]);
        let pj = point(j, x[j]);
        prod *= Ratio::new(-pj, pi - pj);
    }
    prod
}

fn poly_eval(coefs: &Vec<Mpz>, x: &Mpz) -> Mpz {
    let mut y = Mpz::from(0);
    for i in 0..coefs.len() {
        if x == &Mpz::from(0) { continue }
        y += &coefs[i] * x.pow(i as u32 + 1)
    }
    y
}
