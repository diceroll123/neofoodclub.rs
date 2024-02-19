use neofoodclub::modifier::{Modifier, ModifierFlags};
use neofoodclub::nfc::NeoFoodClub;

// Round 8765
const ROUND_DATA: &str = r#"
{"foods":[[5,20,24,21,18,7,34,29,38,8],[26,24,20,36,33,40,5,13,8,25],[5,29,22,31,40,27,30,4,8,19],[35,19,36,5,12,37,6,3,29,30],[28,24,36,17,18,9,1,33,19,3]],"round":8765,"start":"2023-05-05T23:14:57+00:00","changes":[{"t":"2023-05-06T00:17:30+00:00","new":7,"old":5,"arena":1,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":10,"old":8,"arena":3,"pirate":2},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":4},{"t":"2023-05-06T01:09:14+00:00","new":4,"old":3,"arena":4,"pirate":2},{"t":"2023-05-06T01:48:19+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T02:04:11+00:00","new":4,"old":3,"arena":0,"pirate":4},{"t":"2023-05-06T07:29:28+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T09:44:15+00:00","new":5,"old":6,"arena":3,"pirate":3},{"t":"2023-05-06T09:55:08+00:00","new":4,"old":3,"arena":0,"pirate":2},{"t":"2023-05-06T11:11:17+00:00","new":12,"old":11,"arena":0,"pirate":1},{"t":"2023-05-06T16:29:01+00:00","new":11,"old":12,"arena":0,"pirate":1},{"t":"2023-05-06T17:16:30+00:00","new":3,"old":4,"arena":0,"pirate":2},{"t":"2023-05-06T19:16:49+00:00","new":4,"old":5,"arena":2,"pirate":3},{"t":"2023-05-06T19:21:01+00:00","new":6,"old":5,"arena":3,"pirate":3}],"pirates":[[6,11,4,3],[14,15,2,9],[10,16,18,20],[1,12,13,5],[8,19,17,7]],"winners":[3,2,3,2,2],"timestamp":"2023-05-06T23:14:20+00:00","lastChange":"2023-05-06T19:21:01+00:00","currentOdds":[[1,11,3,2,3],[1,13,2,7,13],[1,13,2,4,2],[1,2,10,6,6],[1,13,4,2,4]],"openingOdds":[[1,11,3,2,4],[1,13,2,5,13],[1,13,2,5,2],[1,2,8,5,5],[1,13,3,2,4]]}
"#;

const BET_AMOUNT: u32 = 8000;

fn make_test_nfc() -> NeoFoodClub {
    NeoFoodClub::from_json(ROUND_DATA, Some(BET_AMOUNT), None, None).unwrap()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_getters() {
        let nfc = make_test_nfc();

        assert_eq!(nfc.round(), 8765);
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

        assert_eq!(bets.bets_hash(), "baakfccakpceapaeaa");
    }

    #[test]
    fn test_bustproof_amounts_hash() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert_eq!(
            bets.amounts_hash(),
            Some("AVrCXSAEOAZoAZoBJVAVr".to_string())
        );
    }

    #[test]
    fn test_make_url() {
        let nfc = make_test_nfc();
        let bets = nfc.make_bustproof_bets().unwrap();

        assert_eq!(
            nfc.make_url(&bets),
            "https://neofood.club/#round=8765&b=baakfccakpceapaeaa&a=AVrCXSAEOAZoAZoBJVAVr"
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

        assert!(bets.is_guaranteed_win());
    }

    #[test]
    fn test_is_guaranteed_to_win_false() {
        let nfc = make_test_nfc();
        let bets = nfc.make_crazy_bets();

        assert!(!bets.is_guaranteed_win());
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
        let bets = nfc.make_maxter_bets();

        assert!(!bets.is_crazy());
    }

    #[test]
    fn test_is_gambit_false() {
        let nfc = make_test_nfc();
        let bets = nfc.make_crazy_bets();

        assert!(!bets.is_gambit());

        let bets = nfc.make_bustproof_bets().unwrap();
        assert!(!bets.is_gambit());

        let bets = nfc.make_maxter_bets();
        assert!(!bets.is_gambit());
    }

    #[test]
    fn test_is_gambit_true() {
        let nfc = make_test_nfc();
        let bets = nfc.make_gambit_bets(0x12481);

        assert!(bets.is_gambit());
    }
}
