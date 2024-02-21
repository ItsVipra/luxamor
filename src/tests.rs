#[cfg(test)]
fn benchmark_id_gen() {
    use super::helpers::new_link;
    use std::time::Instant;
    let now = Instant::now();
    for _i in 0..1_000_000 {
        let link = new_link(16);
    }
    println!("Time taken: {:.2?} - average {:.2?}", now.elapsed(), now.elapsed()/1_000_000);

}