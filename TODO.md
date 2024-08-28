# TODO

- update output with workdays results
- add CSV output of distributions
- add CLI parameter to specify which workdays, holidays, calendar config file to use
- switch to BetaPERT distribution:

```rust
use rand::Rng;

fn beta_pert_sample(min: f64, likely: f64, max: f64) -> f64 {
    let alpha1 = 1.0 + 4.0 * (likely - min) / (max - min);
    let alpha2 = 1.0 + 4.0 * (max - likely) / (max - min);

    let mut rng = rand::thread_rng();
    let u: f64 = rng.gen(); // Generate a uniform random number between 0 and 1

    // Inverse transform sampling
    let x = inverse_beta_cdf(u, alpha1, alpha2);

    // Scale and shift to the desired range
    min + (max - min) * x
}

fn inverse_beta_cdf(p: f64, alpha1: f64, alpha2: f64) -> f64 {
    // This is a simple approximation of the inverse beta CDF
    // For more accurate results, consider using a numerical method or a statistics library
    let x = p.powf(1.0 / alpha1) / (p.powf(1.0 / alpha1) + (1.0 - p).powf(1.0 / alpha2));
    x
}

fn main() {
    let min = 10.0;
    let likely = 20.0;
    let max = 40.0;

    let sample = beta_pert_sample(min, likely, max);
    println!("Random sample from BetaPERT({}, {}, {}): {}", min, likely, max, sample);
}
```
