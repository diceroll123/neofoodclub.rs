use neofoodclub::math::{self, BET_AMOUNT_MAX, BET_AMOUNT_MIN};
use neofoodclub::modifier::{Modifier, ModifierFlags};
use neofoodclub::nfc::NeoFoodClub;

// Round 8765
const ROUND_DATA_JSON: &str = r#"
{"foods":[[5,20,24,21,18,7,34,29,38,8],[26,24,20,36,33,40,5,13,8,25],[5,29,22,31,40,27,30,4,8,19],[35,19,36,5,12,37,6,3,29,30],[28,24,36,17,18,9,1,33,19,3]],"round":8765,"start":"2023-05-05T23:14:57+00:00","changes":[{"t":"2023-05-06T00:17:30+00:00","new":7,"old":5,"arena":1,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":10,"old":8,"arena":3,"pirate":2},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":4},{"t":"2023-05-06T01:09:14+00:00","new":4,"old":3,"arena":4,"pirate":2},{"t":"2023-05-06T01:48:19+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T02:04:11+00:00","new":4,"old":3,"arena":0,"pirate":4},{"t":"2023-05-06T07:29:28+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T09:44:15+00:00","new":5,"old":6,"arena":3,"pirate":3},{"t":"2023-05-06T09:55:08+00:00","new":4,"old":3,"arena":0,"pirate":2},{"t":"2023-05-06T11:11:17+00:00","new":12,"old":11,"arena":0,"pirate":1},{"t":"2023-05-06T16:29:01+00:00","new":11,"old":12,"arena":0,"pirate":1},{"t":"2023-05-06T17:16:30+00:00","new":3,"old":4,"arena":0,"pirate":2},{"t":"2023-05-06T19:16:49+00:00","new":4,"old":5,"arena":2,"pirate":3},{"t":"2023-05-06T19:21:01+00:00","new":6,"old":5,"arena":3,"pirate":3}],"pirates":[[6,11,4,3],[14,15,2,9],[10,16,18,20],[1,12,13,5],[8,19,17,7]],"winners":[3,2,3,2,2],"timestamp":"2023-05-06T23:14:20+00:00","lastChange":"2023-05-06T19:21:01+00:00","currentOdds":[[1,11,3,2,3],[1,13,2,7,13],[1,13,2,4,2],[1,2,10,6,6],[1,13,4,2,4]],"openingOdds":[[1,11,3,2,4],[1,13,2,5,13],[1,13,2,5,2],[1,2,8,5,5],[1,13,3,2,4]]}
"#;

/// Round 7956
const ROUND_DATA_URL: &str = r#"/#round=7956&pirates=[[2,8,14,11],[20,7,6,10],[19,4,12,15],[3,1,5,13],[17,16,18,9]]&openingOdds=[[1,2,13,3,5],[1,4,2,4,6],[1,3,13,7,2],[1,13,2,3,3],[1,8,2,4,12]]&currentOdds=[[1,0,13,3,5],[1,4,2,4,6],[1,3,13,7,2],[1,13,2,3,3],[1,8,2,4,13]]&foods=[[26,25,4,9,21,1,33,11,7,10],[12,9,14,35,25,6,21,19,40,37],[17,30,21,39,37,15,29,40,31,10],[10,18,35,9,34,23,27,32,28,12],[11,20,9,33,7,14,4,23,31,26]]&winners=[1,3,4,2,4]&timestamp=2021-02-16T23:47:37+00:00]"#;

const BET_AMOUNT: u32 = 8000;

fn make_test_nfc() -> NeoFoodClub {
    NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None)
}

fn make_test_nfc_from_url() -> NeoFoodClub {
    NeoFoodClub::from_url(ROUND_DATA_URL, Some(BET_AMOUNT), None, None)
}

#[cfg(test)]
mod tests {

    // we parallelize our round data calculations, so nothing is guaranteed to be in order
    // so in our tests we will be sorting and comparing that way

    use core::panic;

    use rayon::prelude::*;

    use super::*;

    #[test]
    fn test_getters() {
        let nfc = make_test_nfc();

        assert_eq!(nfc.round(), 8765);
        assert_eq!(nfc.bet_amount, Some(8000));
    }

    #[test]
    fn test_from_url() {
        let nfc = make_test_nfc_from_url();

        assert_eq!(nfc.round(), 7956);
        assert_eq!(nfc.bet_amount, Some(8000));
    }

    #[test]
    fn test_max_amount_of_bets_10() {
        let mut nfc = make_test_nfc();
        let new_modifier = Modifier::new(ModifierFlags::EMPTY.bits());

        nfc.modifier = new_modifier;

        assert_eq!(nfc.max_amount_of_bets(), 10);
    }

    #[test]
    fn test_max_amount_of_bets_15() {
        let mut nfc = make_test_nfc();
        let new_modifier = Modifier::new(ModifierFlags::CHARITY_CORNER.bits());

        nfc.modifier = new_modifier;

        assert_eq!(nfc.max_amount_of_bets(), 15);
    }

    #[test]
    fn test_bustproof_bets_hash() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        let bets_hash = bets.bets_hash();

        let mut binaries = math::bets_hash_to_bet_binaries(&bets_hash);
        binaries.sort_unstable();

        let expected = [4096, 8192, 16400, 16416, 16448, 16512, 32768];

        assert_eq!(binaries, expected);
    }

    #[test]
    fn test_bustproof_amounts_hash() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        let amounts_hash = bets.amounts_hash();

        let mut bet_amounts = math::amounts_hash_to_bet_amounts(&amounts_hash.unwrap());

        bet_amounts.sort_unstable();

        let expected = [
            Some(1600),
            Some(2461),
            Some(2461),
            Some(2666),
            Some(2666),
            Some(4571),
            Some(8000),
        ];

        assert_eq!(bet_amounts, expected);
    }

    #[test]
    fn test_make_url() {
        // since the order is not guaranteed, we will be using a querystring parser
        // and then comparing the values

        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        let url = nfc.make_url(&bets);

        let [(beginning, round_number), (b, bets_hash), (a, amounts_hash)] =
            querystring::querify(&url)[..]
        else {
            panic!("Failed to parse query strings from URL.");
        };

        assert_eq!(beginning, "https://neofood.club/#round");
        assert_eq!(round_number, nfc.round().to_string());
        assert_eq!(b, "b");
        assert_eq!(a, "a");

        let mut binaries = math::bets_hash_to_bet_binaries(bets_hash);
        binaries.sort_unstable();

        let expected_binaries = [4096, 8192, 16400, 16416, 16448, 16512, 32768];

        assert_eq!(binaries, expected_binaries);

        let mut bet_amounts = math::amounts_hash_to_bet_amounts(amounts_hash);

        bet_amounts.sort_unstable();

        let expected_bet_amounts = [
            Some(1600),
            Some(2461),
            Some(2461),
            Some(2666),
            Some(2666),
            Some(4571),
            Some(8000),
        ];

        assert_eq!(bet_amounts, expected_bet_amounts);
    }

    #[test]
    fn test_get_win_units() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert_eq!(nfc.get_win_units(&bets), 20);
    }

    #[test]
    fn test_get_win_np() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert_eq!(nfc.get_win_np(&bets), 32_000);
    }

    #[test]
    fn test_is_bustproof_true() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert!(bets.is_bustproof());
    }

    #[test]
    fn test_is_bustproof_false() {
        let nfc = make_test_nfc();
        let bets = nfc.make_crazy_bets();

        assert!(!bets.is_bustproof());
    }

    #[test]
    fn test_is_guaranteed_to_win_true() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert!(bets.is_guaranteed_win(&nfc));
    }

    #[test]
    fn test_is_guaranteed_to_win_false() {
        let nfc = make_test_nfc();
        let bets = nfc.make_crazy_bets();

        assert!(!bets.is_guaranteed_win(&nfc));
    }

    #[test]
    fn test_get_winning_pirates() {
        let nfc = make_test_nfc();
        let winners = nfc.winners();

        assert_eq!(winners, [3, 2, 3, 2, 2]);
    }

    #[test]
    fn test_get_winners_binary() {
        let nfc = make_test_nfc();
        let winners = nfc.winners_binary();

        assert_eq!(winners, 148036);
        assert_eq!(winners, 0x24244);
        assert_eq!(winners, 0b100100001001000100);
    }

    #[test]
    fn test_is_over() {
        let nfc = make_test_nfc();
        assert!(nfc.is_over());
    }

    #[test]
    fn test_is_crazy_false() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert!(!bets.is_crazy());
    }

    #[test]
    fn test_is_crazy_true() {
        let nfc = make_test_nfc();
        let bets = nfc.make_crazy_bets();

        assert!(bets.is_crazy());
    }

    #[test]
    fn test_maxter_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_max_ter_bets();

        assert!(!bets.is_crazy());
    }

    #[test]
    fn test_is_gambit_false() {
        let nfc = make_test_nfc();
        let bets = nfc.make_crazy_bets();

        assert!(!bets.is_gambit());

        let bets = nfc.make_bustproof_bets().unwrap();
        assert!(!bets.is_gambit());

        let bets = nfc.make_max_ter_bets();
        assert!(!bets.is_gambit());
    }

    #[test]
    fn test_is_gambit_true() {
        let nfc = make_test_nfc();
        let bets = nfc.make_gambit_bets(0x12481);

        assert!(bets.is_gambit());
    }

    #[test]
    fn test_bet_amounts_hash_encoding_and_decoding() {
        // loop from 50 to 70304 in parallel
        (BET_AMOUNT_MIN..BET_AMOUNT_MAX)
            .into_par_iter() // makes this go from like 1.75s like no time
            .for_each(|amount| {
                let amounts = vec![Some(amount); 10];
                let hash = math::bet_amounts_to_amounts_hash(&amounts);
                assert_eq!(
                    math::amounts_hash_to_bet_amounts(&hash),
                    vec![Some(amount); 10]
                );
            });
    }

    #[test]
    fn test_bet_amounts_hash_encoding_and_decoding_none() {
        // amount too low, returns None
        let amounts = vec![Some(BET_AMOUNT_MIN - 1); 10];
        let hash = math::bet_amounts_to_amounts_hash(&amounts);
        assert_eq!(math::amounts_hash_to_bet_amounts(&hash), vec![None; 10]);
    }

    #[test]
    fn test_winning_pirates_from_url() {
        let nfc = make_test_nfc_from_url();

        assert_eq!(nfc.winners(), [1, 3, 4, 2, 4]);
    }
}
