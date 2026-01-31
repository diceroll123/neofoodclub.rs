use neofoodclub::math;
use neofoodclub::modifier::{Modifier, ModifierFlags};
use neofoodclub::nfc::{NeoFoodClub, ProbabilityModel};

fn main() {
    divan::main();
}

// Test data from integration tests
const ROUND_DATA_JSON: &str = r#"
{"foods":[[5,20,24,21,18,7,34,29,38,8],[26,24,20,36,33,40,5,13,8,25],[5,29,22,31,40,27,30,4,8,19],[35,19,36,5,12,37,6,3,29,30],[28,24,36,17,18,9,1,33,19,3]],"round":8765,"start":"2023-05-05T23:14:57+00:00","changes":[{"t":"2023-05-06T00:17:30+00:00","new":7,"old":5,"arena":1,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":10,"old":8,"arena":3,"pirate":2},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":4},{"t":"2023-05-06T01:09:14+00:00","new":4,"old":3,"arena":4,"pirate":2},{"t":"2023-05-06T01:48:19+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T02:04:11+00:00","new":4,"old":3,"arena":0,"pirate":4},{"t":"2023-05-06T07:29:28+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T09:44:15+00:00","new":5,"old":6,"arena":3,"pirate":3},{"t":"2023-05-06T09:55:08+00:00","new":4,"old":3,"arena":0,"pirate":2},{"t":"2023-05-06T11:11:17+00:00","new":12,"old":11,"arena":0,"pirate":1},{"t":"2023-05-06T16:29:01+00:00","new":11,"old":12,"arena":0,"pirate":1},{"t":"2023-05-06T17:16:30+00:00","new":3,"old":4,"arena":0,"pirate":2},{"t":"2023-05-06T19:16:49+00:00","new":4,"old":5,"arena":2,"pirate":3},{"t":"2023-05-06T19:21:01+00:00","new":6,"old":5,"arena":3,"pirate":3}],"pirates":[[6,11,4,3],[14,15,2,9],[10,16,18,20],[1,12,13,5],[8,19,17,7]],"winners":[3,2,3,2,2],"timestamp":"2023-05-06T23:14:20+00:00","lastChange":"2023-05-06T19:21:01+00:00","currentOdds":[[1,11,3,2,3],[1,13,2,7,13],[1,13,2,4,2],[1,2,10,6,6],[1,13,4,2,4]],"openingOdds":[[1,11,3,2,4],[1,13,2,5,13],[1,13,2,5,2],[1,2,8,5,5],[1,13,3,2,4]]}
"#;

const ROUND_DATA_URL: &str = r#"/#round=7956&pirates=[[2,8,14,11],[20,7,6,10],[19,4,12,15],[3,1,5,13],[17,16,18,9]]&openingOdds=[[1,2,13,3,5],[1,4,2,4,5],[1,3,13,7,2],[1,13,2,3,3],[1,12,2,6,13]]&currentOdds=[[1,2,13,3,5],[1,4,2,4,6],[1,3,13,7,2],[1,13,2,3,3],[1,8,2,4,12]]&foods=[[26,25,4,9,21,1,33,11,7,10],[12,9,14,35,25,6,21,19,40,37],[17,30,21,39,37,15,29,40,31,10],[10,18,35,9,34,23,27,32,28,12],[11,20,9,33,7,14,4,23,31,26]]&winners=[1,3,4,2,4]&timestamp=2021-02-16T23:47:37+00:00"#;

const BET_AMOUNT: u32 = 8000;

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

// NeoFoodClub operation benchmarks

#[divan::bench]
fn bench_nfc_from_json() {
    divan::black_box(NeoFoodClub::from_json(
        divan::black_box(ROUND_DATA_JSON),
        Some(BET_AMOUNT),
        None,
        None,
    ));
}

#[divan::bench]
fn bench_nfc_from_url() {
    divan::black_box(NeoFoodClub::from_url(
        divan::black_box(ROUND_DATA_URL),
        Some(BET_AMOUNT),
        None,
        None,
    ));
}

#[divan::bench]
fn bench_nfc_probabilities() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.probabilities());
}

#[divan::bench]
fn bench_nfc_probabilities_logit() {
    let nfc = NeoFoodClub::from_json(
        ROUND_DATA_JSON,
        Some(BET_AMOUNT),
        Some(ProbabilityModel::MultinomialLogitModel),
        None,
    );
    divan::black_box(nfc.probabilities());
}

#[divan::bench]
fn bench_nfc_round_dict_data() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.round_dict_data());
}

#[divan::bench]
fn bench_nfc_get_arenas() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.get_arenas());
}

#[divan::bench]
fn bench_nfc_get_arenas_positives() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.get_arenas().positives());
}

// Bet generation benchmarks

#[divan::bench]
fn bench_make_max_ter_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_max_ter_bets());
}

#[divan::bench]
fn bench_make_bustproof_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_bustproof_bets());
}

#[divan::bench]
fn bench_make_best_gambit_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_best_gambit_bets());
}

#[divan::bench]
fn bench_make_gambit_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_gambit_bets(divan::black_box(0x12481)));
}

#[divan::bench]
fn bench_make_random_gambit_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_random_gambit_bets());
}

#[divan::bench]
fn bench_make_winning_gambit_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_winning_gambit_bets());
}

#[divan::bench]
fn bench_make_random_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_random_bets());
}

#[divan::bench]
fn bench_make_crazy_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_crazy_bets());
}

#[divan::bench]
fn bench_make_all_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_all_bets());
}

#[divan::bench]
fn bench_make_all_max_ter_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_all_max_ter_bets());
}

#[divan::bench]
fn bench_make_tenbet_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_tenbet_bets(divan::black_box(0x88800)).unwrap());
}

#[divan::bench]
fn bench_make_units_bets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(nfc.make_units_bets(divan::black_box(20)));
}

#[divan::bench]
fn bench_make_bets_from_hash() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    divan::black_box(
        nfc.make_bets_from_hash(divan::black_box("ltqvqwgimhqtvrnywrwvijwnn"))
            .unwrap(),
    );
}

#[divan::bench]
fn bench_make_bets_from_binaries() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let binaries = vec![0x80000, 0x8000, 0x800, 0x80, 0x8];
    divan::black_box(nfc.make_bets_from_binaries(divan::black_box(binaries)));
}

#[divan::bench]
fn bench_make_bets_from_indices() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let indices = vec![[1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 1, 0, 0]];
    divan::black_box(nfc.make_bets_from_indices(divan::black_box(indices)));
}

// Bet calculation benchmarks

#[divan::bench]
fn bench_get_win_units() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(nfc.get_win_units(divan::black_box(&bets)));
}

#[divan::bench]
fn bench_get_win_np() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(nfc.get_win_np(divan::black_box(&bets)));
}

#[divan::bench]
fn bench_expected_return() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(bets.expected_return(divan::black_box(&nfc)));
}

#[divan::bench]
fn bench_net_expected() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(bets.net_expected(divan::black_box(&nfc)));
}

#[divan::bench]
fn bench_fill_bet_amounts() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let mut bets = nfc.make_all_bets();
    bets.bet_amounts = None;
    divan::black_box(bets.fill_bet_amounts(divan::black_box(&nfc)));
}

// Bet operation benchmarks

#[divan::bench]
fn bench_is_bustproof() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_bustproof_bets().unwrap();
    divan::black_box(bets.is_bustproof());
}

#[divan::bench]
fn bench_is_gambit() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_best_gambit_bets();
    divan::black_box(bets.is_gambit());
}

#[divan::bench]
fn bench_is_crazy() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_crazy_bets();
    divan::black_box(bets.is_crazy());
}

#[divan::bench]
fn bench_is_tenbet() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_tenbet_bets(0x88800).unwrap();
    divan::black_box(bets.is_tenbet());
}

#[divan::bench]
fn bench_is_guaranteed_win() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_bustproof_bets().unwrap();
    divan::black_box(bets.is_guaranteed_win(divan::black_box(&nfc)));
}

#[divan::bench]
fn bench_count_tenbets() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_tenbet_bets(0x88800).unwrap();
    divan::black_box(bets.count_tenbets());
}

// URL and hash benchmarks

#[divan::bench]
fn bench_make_url() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(nfc.make_url(
        Some(divan::black_box(&bets)),
        divan::black_box(true),
        divan::black_box(false),
    ));
}

#[divan::bench]
fn bench_bets_hash() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(bets.bets_hash());
}

#[divan::bench]
fn bench_amounts_hash() {
    let nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let bets = nfc.make_max_ter_bets();
    divan::black_box(bets.amounts_hash());
}

// Modifier benchmarks

#[divan::bench]
fn bench_nfc_with_modifier_reverse() {
    let mut nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let modifier = Modifier::new(ModifierFlags::REVERSE.bits(), None, None).unwrap();
    divan::black_box(nfc.with_modifier(divan::black_box(modifier)));
}

#[divan::bench]
fn bench_nfc_with_modifier_opening_odds() {
    let mut nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let modifier = Modifier::new(ModifierFlags::OPENING_ODDS.bits(), None, None).unwrap();
    divan::black_box(nfc.with_modifier(divan::black_box(modifier)));
}

#[divan::bench]
fn bench_nfc_with_modifier_charity_corner() {
    let mut nfc = NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None);
    let modifier = Modifier::new(ModifierFlags::CHARITY_CORNER.bits(), None, None).unwrap();
    divan::black_box(nfc.with_modifier(divan::black_box(modifier)));
}
