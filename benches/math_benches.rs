use neofoodclub::math;

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_pirate_binary() {
    divan::black_box(math::pirate_binary(
        divan::black_box(3),
        divan::black_box(2),
    ));
}

#[divan::bench]
fn bench_pirates_binary() {
    divan::black_box(math::pirates_binary(divan::black_box([1, 2, 3, 4, 1])));
}

#[divan::bench]
fn bench_binary_to_indices() {
    divan::black_box(math::binary_to_indices(divan::black_box(0x48212)));
}

#[divan::bench]
fn bench_bets_hash_to_bet_indices_small() {
    divan::black_box(math::bets_hash_to_bet_indices(divan::black_box("faa")).unwrap());
}

#[divan::bench]
fn bench_bets_hash_to_bet_indices_medium() {
    divan::black_box(
        math::bets_hash_to_bet_indices(divan::black_box("jmbcoemycobmbhofmdcoamyck")).unwrap(),
    );
}

#[divan::bench]
fn bench_bets_hash_to_bet_indices_large() {
    divan::black_box(
        math::bets_hash_to_bet_indices(divan::black_box("dgpqsxgtqsigqqsngrqsegpvsdgfqqsgsqsdgk"))
            .unwrap(),
    );
}

#[divan::bench]
fn bench_bets_hash_to_bets_count() {
    divan::black_box(
        math::bets_hash_to_bets_count(divan::black_box("dgpqsxgtqsigqqsngrqsegpvsdgfqqsgsqsdgk"))
            .unwrap(),
    );
}

#[divan::bench]
fn bench_bet_amounts_to_amounts_hash() {
    let amounts = vec![Some(50), Some(100), Some(150), Some(200), Some(250)];
    divan::black_box(math::bet_amounts_to_amounts_hash(divan::black_box(
        &amounts,
    )));
}

#[divan::bench]
fn bench_amounts_hash_to_bet_amounts() {
    divan::black_box(
        math::amounts_hash_to_bet_amounts(divan::black_box("EmxCoKCoKCglDKUCYqEXkByWBpqzGO"))
            .unwrap(),
    );
}

#[divan::bench]
fn bench_bets_hash_to_bet_binaries() {
    divan::black_box(
        math::bets_hash_to_bet_binaries(divan::black_box("ltqvqwgimhqtvrnywrwvijwnn")).unwrap(),
    );
}

#[divan::bench]
fn bench_bets_hash_value() {
    let indices = vec![[1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 1, 0, 0]];
    divan::black_box(math::bets_hash_value(divan::black_box(indices)));
}

#[divan::bench]
fn bench_expand_ib_object() {
    let bets = vec![
        [1, 4, 2, 2, 0],
        [1, 0, 2, 2, 4],
        [0, 4, 2, 2, 4],
        [4, 0, 2, 2, 4],
        [0, 1, 2, 2, 0],
    ];
    let bet_odds = vec![13, 26, 52, 13, 26];
    divan::black_box(math::expand_ib_object(
        divan::black_box(&bets),
        divan::black_box(&bet_odds),
    ));
}

#[divan::bench]
fn bench_make_round_dicts() {
    let stds = [
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
    ];
    let odds = [
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 5],
    ];
    divan::black_box(math::make_round_dicts(
        divan::black_box(stds),
        divan::black_box(odds),
    ));
}

#[divan::bench]
fn bench_build_chance_objects() {
    let bets = vec![[1, 4, 2, 2, 0], [1, 0, 2, 2, 4], [0, 4, 2, 2, 4]];
    let bet_odds = vec![13, 26, 52];
    let probabilities = [
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
        [0.0, 0.25, 0.25, 0.25, 0.25],
    ];
    divan::black_box(math::build_chance_objects(
        divan::black_box(&bets),
        divan::black_box(&bet_odds),
        divan::black_box(probabilities),
    ));
}
