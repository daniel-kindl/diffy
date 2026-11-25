use std::time::Instant;
fn main() {
    for f in std::env::args().skip(1) {
        let l = std::fs::read(format!("{f}.left.txt")).unwrap();
        let r = std::fs::read(format!("{f}.right.txt")).unwrap();
        let input = gix_imara_diff::InternedInput::new(&l[..], &r[..]);
        let mut best = f64::MAX;
        for _ in 0..5 {
            let t = Instant::now();
            let mut d = gix_imara_diff::Diff::compute(gix_imara_diff::Algorithm::Histogram, &input);
            d.postprocess_lines(&input);
            best = best.min(t.elapsed().as_secs_f64() * 1000.0);
        }
        println!("{f}: best of 5 {best:.1} ms");
    }
}
