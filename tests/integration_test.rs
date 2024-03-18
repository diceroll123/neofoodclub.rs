use neofoodclub::math::{self, BET_AMOUNT_MAX, BET_AMOUNT_MIN};
use neofoodclub::modifier::{Modifier, ModifierFlags};
use neofoodclub::nfc::NeoFoodClub;

// Round 8765
const ROUND_DATA_JSON: &str = r#"
{"foods":[[5,20,24,21,18,7,34,29,38,8],[26,24,20,36,33,40,5,13,8,25],[5,29,22,31,40,27,30,4,8,19],[35,19,36,5,12,37,6,3,29,30],[28,24,36,17,18,9,1,33,19,3]],"round":8765,"start":"2023-05-05T23:14:57+00:00","changes":[{"t":"2023-05-06T00:17:30+00:00","new":7,"old":5,"arena":1,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":10,"old":8,"arena":3,"pirate":2},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":4},{"t":"2023-05-06T01:09:14+00:00","new":4,"old":3,"arena":4,"pirate":2},{"t":"2023-05-06T01:48:19+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T02:04:11+00:00","new":4,"old":3,"arena":0,"pirate":4},{"t":"2023-05-06T07:29:28+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T09:44:15+00:00","new":5,"old":6,"arena":3,"pirate":3},{"t":"2023-05-06T09:55:08+00:00","new":4,"old":3,"arena":0,"pirate":2},{"t":"2023-05-06T11:11:17+00:00","new":12,"old":11,"arena":0,"pirate":1},{"t":"2023-05-06T16:29:01+00:00","new":11,"old":12,"arena":0,"pirate":1},{"t":"2023-05-06T17:16:30+00:00","new":3,"old":4,"arena":0,"pirate":2},{"t":"2023-05-06T19:16:49+00:00","new":4,"old":5,"arena":2,"pirate":3},{"t":"2023-05-06T19:21:01+00:00","new":6,"old":5,"arena":3,"pirate":3}],"pirates":[[6,11,4,3],[14,15,2,9],[10,16,18,20],[1,12,13,5],[8,19,17,7]],"winners":[3,2,3,2,2],"timestamp":"2023-05-06T23:14:20+00:00","lastChange":"2023-05-06T19:21:01+00:00","currentOdds":[[1,11,3,2,3],[1,13,2,7,13],[1,13,2,4,2],[1,2,10,6,6],[1,13,4,2,4]],"openingOdds":[[1,11,3,2,4],[1,13,2,5,13],[1,13,2,5,2],[1,2,8,5,5],[1,13,3,2,4]]}
"#;

/// Round 7956
const ROUND_DATA_URL: &str = r#"/#round=7956&pirates=[[2,8,14,11],[20,7,6,10],[19,4,12,15],[3,1,5,13],[17,16,18,9]]&openingOdds=[[1,2,13,3,5],[1,4,2,4,5],[1,3,13,7,2],[1,13,2,3,3],[1,12,2,6,13]]&currentOdds=[[1,2,13,3,5],[1,4,2,4,6],[1,3,13,7,2],[1,13,2,3,3],[1,8,2,4,12]]&foods=[[26,25,4,9,21,1,33,11,7,10],[12,9,14,35,25,6,21,19,40,37],[17,30,21,39,37,15,29,40,31,10],[10,18,35,9,34,23,27,32,28,12],[11,20,9,33,7,14,4,23,31,26]]&winners=[1,3,4,2,4]&timestamp=2021-02-16T23:47:37+00:00"#;

const BET_AMOUNT: u32 = 8000;

fn make_test_nfc() -> NeoFoodClub {
    NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, None)
}

fn make_test_nfc_with_modifier(modifier: Modifier) -> NeoFoodClub {
    NeoFoodClub::from_json(ROUND_DATA_JSON, Some(BET_AMOUNT), None, Some(modifier))
}

fn make_test_nfc_from_url() -> NeoFoodClub {
    NeoFoodClub::from_url(ROUND_DATA_URL, Some(BET_AMOUNT), None, None)
}

#[cfg(test)]
mod tests {

    // we parallelize our round data calculations, so nothing is guaranteed to be in order
    // so in our tests we will be sorting and comparing that way

    use core::panic;
    use std::collections::HashMap;

    use chrono::{DateTime, NaiveTime, TimeDelta};
    use neofoodclub::{bets::BetAmounts, pirates::PartialPirateThings};
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
        let new_modifier = Modifier::new(ModifierFlags::EMPTY.bits(), None, None);

        nfc.modifier = new_modifier;

        assert_eq!(nfc.max_amount_of_bets(), 10);
    }

    #[test]
    fn test_max_amount_of_bets_15() {
        let mut nfc = make_test_nfc();
        let new_modifier = Modifier::new(ModifierFlags::CHARITY_CORNER.bits(), None, None);

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

        let url = nfc.make_url(&bets, true, false);

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
    fn test_make_url_from_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert_eq!(
            nfc.make_url(&bets, true, false),
            bets.make_url(&nfc, true, false)
        );
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
    fn test_get_win_np_from_url() {
        let nfc = make_test_nfc_from_url();
        let bets = nfc.make_bets_from_hash("aukacfukycuulacauutcbukdc");

        assert_eq!(nfc.get_win_np(&bets), 192_000);
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

    #[test]
    fn test_bet_hash_encoding() {
        let crazy_hash = "ltqvqwgimhqtvrnywrwvijwnn";

        let nfc = make_test_nfc();

        let bets = nfc.make_bets_from_hash(crazy_hash);

        assert_eq!(bets.bets_hash(), crazy_hash);
    }

    #[test]
    fn test_bet_amount_setting() {
        let mut nfc = make_test_nfc();
        nfc.bet_amount = Some(1000);

        assert_eq!(nfc.bet_amount, Some(1000));
    }

    #[test]
    fn test_bet_amount_setting_with_bets() {
        let mut nfc = make_test_nfc();
        nfc.bet_amount = Some(1000);

        let bets = nfc.make_winning_gambit_bets().unwrap();

        assert_eq!(bets.bet_amounts, Some(vec![Some(1000); 10]));
    }

    #[test]
    fn test_arena_ratio() {
        let nfc = make_test_nfc();

        let ratio = nfc.arenas.get_arena(0).unwrap().ratio();

        assert!(ratio < 0.0);
    }

    #[test]
    fn test_arena_is_negative() {
        let nfc = make_test_nfc();

        let arena = nfc.arenas.get_arena(0).unwrap();

        assert!(arena.is_negative());
    }

    #[test]
    fn test_arena_name() {
        let nfc = make_test_nfc();

        let arena = nfc.arenas.get_arena(0).unwrap();

        assert_eq!(arena.get_name(), "Shipwreck");
    }

    #[test]
    fn test_arena_ids() {
        let nfc = make_test_nfc();

        let arena = nfc.arenas.get_arena(0).unwrap();

        assert_eq!(arena.ids(), [6, 11, 4, 3]);
    }

    #[test]
    fn test_arena_get_pirate_by_index() {
        let nfc = make_test_nfc();

        let arena = nfc.arenas.get_arena(0).unwrap();

        let pirate = arena.get_pirate_by_index(0).unwrap();

        assert_eq!(pirate.id, 6);
    }

    #[test]
    fn test_arenas_get_pirate_by_id() {
        let nfc = make_test_nfc();

        let pirate = nfc.arenas.get_pirate_by_id(1).unwrap();

        assert_eq!(pirate.get_name(), "Dan");
    }

    #[test]
    fn test_arenas_get_pirates_by_id() {
        let nfc = make_test_nfc();

        let pirates = nfc.arenas.get_pirates_by_id(&[1, 2, 3]);

        assert_eq!(pirates[0].get_name(), "Dan");
        assert_eq!(pirates[1].get_name(), "Sproggie");
        assert_eq!(pirates[2].get_name(), "Orvinn");
    }

    #[test]
    fn test_arenas_get_all_pirates_flat() {
        let nfc = make_test_nfc();

        let pirates = nfc.arenas.get_all_pirates_flat();

        assert_eq!(pirates.len(), 20);
    }

    #[test]
    fn test_arenas_get_pirates_from_binary() {
        let nfc = make_test_nfc();

        let pirates = nfc.arenas.get_pirates_from_binary(0x12480);

        assert_eq!(pirates.len(), 4);

        assert_eq!(pirates[0].get_name(), "Orvinn");
        assert_eq!(pirates[1].get_name(), "Sproggie");
        assert_eq!(pirates[2].get_name(), "Franchisco");
        assert_eq!(pirates[3].get_name(), "Dan");
    }

    #[test]
    fn test_arenas_get_all_pirates() {
        let nfc = make_test_nfc();

        let pirates = nfc.arenas.get_all_pirates();

        assert_eq!(pirates.len(), 5);
    }

    #[test]
    fn test_arenas_best() {
        let nfc = make_test_nfc();

        let best = nfc.arenas.best();

        assert_eq!(best[0].get_name(), "Lagoon");
        assert_eq!(best[1].get_name(), "Hidden");
        assert_eq!(best[2].get_name(), "Harpoon");
        assert_eq!(best[3].get_name(), "Shipwreck");
        assert_eq!(best[4].get_name(), "Treasure");
    }

    #[test]
    fn test_arenas_pirate_ids() {
        let nfc = make_test_nfc();

        let ids = nfc.arenas.pirate_ids();

        assert_eq!(ids[0], &[6, 11, 4, 3]);
    }

    #[test]
    fn test_partial_pirate_get_image() {
        let nfc = make_test_nfc();

        let pirate = nfc.arenas.get_pirate_by_id(1).unwrap();

        assert_eq!(
            pirate.get_image(),
            "http://images.neopets.com/pirates/fc/fc_pirate_1.gif"
        );
    }

    #[test]
    fn test_pirate_positive_foods() {
        let nfc = make_test_nfc();

        let pirate = nfc.arenas.get_pirate_by_id(1).unwrap();

        let foods = pirate.positive_foods(&nfc).unwrap();

        assert_eq!(foods, [12, 6]);
    }

    #[test]
    fn test_pirate_positive_foods_none() {
        let nfc = make_test_nfc();

        let pirate = nfc.arenas.get_pirate_by_id(4).unwrap();

        let foods = pirate.positive_foods(&nfc);

        assert_eq!(foods, None);
    }

    #[test]
    fn test_pirate_negative_foods_none() {
        let nfc = make_test_nfc();

        let pirate = nfc.arenas.get_pirate_by_id(1).unwrap();

        let foods = pirate.negative_foods(&nfc);

        assert_eq!(foods, None);
    }

    #[test]
    fn test_pirate_negative_foods() {
        let nfc = make_test_nfc();

        let pirate = nfc.arenas.get_pirate_by_id(2).unwrap();

        let foods = pirate.negative_foods(&nfc).unwrap();

        assert_eq!(foods, [40, 25]);
    }

    #[test]
    fn test_bets_hash_to_bets_count() {
        let bets_hash = "aukacfukycuulacauutcbukdc";
        let bets = math::bets_hash_to_bets_count(bets_hash);

        assert_eq!(bets, 10);
    }

    #[test]
    fn test_bets_indices_to_bet_binaries() {
        let bins = neofoodclub::math::bets_indices_to_bet_binaries(vec![
            [1, 0, 0, 0, 0],
            [0, 1, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 1, 0],
            [0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0],
        ]);
        assert_eq!(bins, vec![0x80000, 0x8000, 0x800, 0x80, 0x8, 0x80000]);
    }

    #[test]
    fn test_make_best_gambit_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_best_gambit_bets();

        assert!(bets.is_gambit());
    }

    #[test]
    fn test_make_random_gambit_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_random_gambit_bets();

        assert!(bets.is_gambit());
    }

    #[test]
    fn test_make_random_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_random_bets();

        assert_eq!(bets.len(), nfc.max_amount_of_bets());
    }

    #[test]
    fn test_make_all_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_all_bets();

        assert_eq!(bets.len(), 3124);
    }

    #[test]
    #[should_panic]
    fn test_make_gambit_bets_broken() {
        let nfc = make_test_nfc();
        nfc.make_gambit_bets(0x12480);
    }

    #[test]
    fn test_make_tenbet_bets() {
        let nfc = make_test_nfc();
        let bets = nfc.make_tenbet_bets(0x88800);

        assert_eq!(bets.unwrap().len(), 10);
    }

    #[test]
    fn test_bets_is_empty() {
        let nfc = make_test_nfc();
        let bets = nfc.make_tenbet_bets(0x88800);

        assert!(!bets.unwrap().is_empty());
    }

    #[test]
    fn test_bets_get_binaries() {
        let nfc = make_test_nfc();
        let bets = nfc.make_tenbet_bets(0x88800);

        let binaries = bets.unwrap().get_binaries();

        assert_eq!(binaries.len(), 10);
    }

    #[test]
    fn test_nfc_winning_pirates() {
        let nfc = make_test_nfc();
        let pirates = nfc.winning_pirates().unwrap();

        assert_eq!(pirates.len(), 5);
    }

    #[test]
    fn test_make_tenbet_bets_zero_pirates() {
        let nfc = make_test_nfc();
        assert!(nfc.make_tenbet_bets(0).is_err());
    }

    #[test]
    fn test_make_tenbet_bets_too_many_pirates() {
        let nfc = make_test_nfc();
        assert!(nfc.make_tenbet_bets(0x8888888).is_err());
    }

    #[test]
    fn test_bets_expected_return() {
        let nfc = make_test_nfc();
        let bets = nfc.make_max_ter_bets();

        assert!(bets.expected_return(&nfc) > 17.0);
    }

    #[test]
    fn test_bets_net_expected() {
        let nfc = make_test_nfc();
        let bets = nfc.make_max_ter_bets();

        assert!(bets.net_expected(&nfc) > 56316.0);
    }

    #[test]
    fn test_bets_net_expected_no_bet_amount() {
        let mut nfc = make_test_nfc();
        nfc.bet_amount = None;
        let bets = nfc.make_max_ter_bets();

        assert_eq!(bets.net_expected(&nfc), 0.00);
    }

    #[test]
    fn test_bets_set_bet_amounts() {
        let nfc = make_test_nfc();
        let mut bets = nfc.make_max_ter_bets();

        let amounts = neofoodclub::bets::BetAmounts::from_amount(8000, bets.len());
        bets.set_bet_amounts(&Some(amounts));

        assert_eq!(bets.bet_amounts, Some(vec![Some(8000); 10]));
    }

    #[test]
    fn test_bets_set_bet_amounts_zero() {
        let nfc = make_test_nfc();
        let mut bets = nfc.make_max_ter_bets();

        let amounts = neofoodclub::bets::BetAmounts::from_amount(0, bets.len());
        bets.set_bet_amounts(&Some(amounts));

        assert_eq!(bets.bet_amounts, None);
    }

    #[test]
    fn test_bets_set_bet_amounts_zero_length() {
        assert_eq!(
            neofoodclub::bets::BetAmounts::from_amount(8000, 0),
            BetAmounts::None
        );
    }

    #[test]
    fn test_betamounts_to_vec_with_hash() {
        let amounts =
            neofoodclub::bets::BetAmounts::AmountHash("EmxCoKCoKCglDKUCYqEXkByWBpqzGO".to_owned());
        assert_eq!(
            amounts.to_vec(),
            Some(vec![
                Some(11463),
                Some(6172),
                Some(6172),
                Some(5731),
                Some(10030),
                Some(8024),
                Some(13374),
                Some(4000),
                Some(3500),
            ])
        );
    }

    #[test]
    #[should_panic]
    fn test_amounts_hash_to_bet_amounts_invalid() {
        math::amounts_hash_to_bet_amounts("🎲");
    }

    #[test]
    #[should_panic]
    fn test_bets_hash_to_bets_count_invalid() {
        math::bets_hash_to_bets_count("🎲");
    }

    #[test]
    fn test_make_bets_from_binaries_with_duplicate() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bets_from_binaries(vec![0x80000, 0x8000, 0x800, 0x80, 0x8, 0x80000]);

        assert_eq!(bets.len(), 5);
    }

    #[test]
    fn test_make_bets_from_indices() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bets_from_indices(vec![[0, 1, 2, 3, 4]]);

        assert_eq!(bets.len(), 1);
    }

    #[test]
    fn test_nfc_copy() {
        let nfc = make_test_nfc();
        let new_nfc = nfc.copy();

        assert_eq!(nfc.round(), new_nfc.round());
    }

    #[test]
    fn test_max_ter_reverse() {
        let mut nfc = make_test_nfc_from_url();

        nfc.modifier = Modifier::new(ModifierFlags::REVERSE.bits(), None, None);
        let bets = nfc.make_max_ter_bets();

        assert_eq!(
            bets.bet_amounts,
            Some(vec![
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
                Some(8000),
            ]),
        );
    }

    #[test]
    fn test_make_units_bets_20() {
        let nfc = make_test_nfc();
        let bets = nfc.make_units_bets(20);

        for odd in bets.unwrap().odds_values(&nfc) {
            assert!(odd >= 20);
        }
    }

    #[test]
    fn test_make_units_bets_100() {
        let nfc = make_test_nfc();
        let bets = nfc.make_units_bets(100);

        for odd in bets.unwrap().odds_values(&nfc) {
            assert!(odd >= 100);
        }
    }

    #[test]
    fn test_make_units_bets_300000() {
        let nfc = make_test_nfc();
        let bets = nfc.make_units_bets(300_000);

        assert!(bets.is_none());
    }

    #[test]
    fn test_datetime() {
        let nfc = make_test_nfc();
        let start = nfc.start().unwrap();

        let dt = chrono::DateTime::parse_from_rfc3339(&start)
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert!(dt < chrono::Utc::now());
    }

    #[test]
    fn test_get_dst_offset_positive() {
        let dst_mar_2024 = DateTime::parse_from_rfc3339("2024-03-11T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let offset = neofoodclub::utils::get_dst_offset(dst_mar_2024);

        assert_eq!(offset, TimeDelta::try_hours(1).unwrap());
    }

    #[test]
    fn test_get_dst_offset_negative() {
        let dst_nov_2024 = DateTime::parse_from_rfc3339("2024-11-04T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let offset = neofoodclub::utils::get_dst_offset(dst_nov_2024);

        assert_eq!(offset, TimeDelta::try_hours(-1).unwrap());
    }

    #[test]
    fn test_get_dst_offset_zero() {
        let jan_first_2024 = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        let offset = neofoodclub::utils::get_dst_offset(jan_first_2024);

        assert!(offset.is_zero());
    }

    #[test]
    fn test_modifier_custom_odds() {
        let mut custom_odds = HashMap::<u8, u8>::new();
        for id in 1..=20 {
            custom_odds.insert(id, 13);
        }

        let modifier = Modifier::new(ModifierFlags::EMPTY.bits(), Some(custom_odds), None);
        let nfc = make_test_nfc_with_modifier(modifier);

        assert_eq!(
            nfc.current_odds(),
            [
                [1, 13, 13, 13, 13],
                [1, 13, 13, 13, 13],
                [1, 13, 13, 13, 13],
                [1, 13, 13, 13, 13],
                [1, 13, 13, 13, 13]
            ]
        );
    }

    #[test]
    fn test_modifier_custom_time() {
        let control_nfc = make_test_nfc();

        let time = NaiveTime::parse_from_str("12:00:00", "%H:%M:%S").unwrap();

        let modifier = Modifier::new(ModifierFlags::EMPTY.bits(), None, Some(time));

        let nfc = make_test_nfc_with_modifier(modifier);

        let modified_length = nfc.changes().unwrap().len();

        let control_length = control_nfc.changes().unwrap().len();

        assert_ne!(modified_length, control_length);
    }

    #[test]
    fn test_modifier_custom_time_expect_no_changes() {
        let time = NaiveTime::parse_from_str("16:15:00", "%H:%M:%S").unwrap();

        let modifier = Modifier::new(ModifierFlags::EMPTY.bits(), None, Some(time));

        let nfc = make_test_nfc_with_modifier(modifier);

        assert!(nfc.changes().is_none());
    }

    #[test]
    fn test_modifier_custom_time_expect_4_changes() {
        let time = NaiveTime::parse_from_str("18:00:00", "%H:%M:%S").unwrap();

        let modifier = Modifier::new(ModifierFlags::EMPTY.bits(), None, Some(time));

        let nfc = make_test_nfc_with_modifier(modifier);

        assert_eq!(nfc.changes().unwrap().len(), 4);
    }

    #[test]
    fn test_modifier_custom_time_expect_14_changes() {
        let time = NaiveTime::parse_from_str("12:20:00", "%H:%M:%S").unwrap();

        let modifier = Modifier::new(ModifierFlags::EMPTY.bits(), None, Some(time));

        let nfc = make_test_nfc_with_modifier(modifier);

        assert_eq!(nfc.changes().unwrap().len(), 14);
    }
}
