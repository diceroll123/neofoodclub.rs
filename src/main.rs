use nfc::NeoFoodClub;

pub mod arena;
pub mod bets;
pub mod chance;
pub mod food_adjustments;
pub mod math;
pub mod models;
pub mod modifier;
pub mod nfc;
pub mod odds;
pub mod pirates;
pub mod utils;

fn main() {
    let data = r#"
    {"foods":[[5,20,24,21,18,7,34,29,38,8],[26,24,20,36,33,40,5,13,8,25],[5,29,22,31,40,27,30,4,8,19],[35,19,36,5,12,37,6,3,29,30],[28,24,36,17,18,9,1,33,19,3]],"round":8765,"start":"2023-05-05T23:14:57+00:00","changes":[{"t":"2023-05-06T00:17:30+00:00","new":7,"old":5,"arena":1,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":10,"old":8,"arena":3,"pirate":2},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":3},{"t":"2023-05-06T00:21:43+00:00","new":6,"old":5,"arena":3,"pirate":4},{"t":"2023-05-06T01:09:14+00:00","new":4,"old":3,"arena":4,"pirate":2},{"t":"2023-05-06T01:48:19+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T02:04:11+00:00","new":4,"old":3,"arena":0,"pirate":4},{"t":"2023-05-06T07:29:28+00:00","new":3,"old":4,"arena":0,"pirate":4},{"t":"2023-05-06T09:44:15+00:00","new":5,"old":6,"arena":3,"pirate":3},{"t":"2023-05-06T09:55:08+00:00","new":4,"old":3,"arena":0,"pirate":2},{"t":"2023-05-06T11:11:17+00:00","new":12,"old":11,"arena":0,"pirate":1},{"t":"2023-05-06T16:29:01+00:00","new":11,"old":12,"arena":0,"pirate":1},{"t":"2023-05-06T17:16:30+00:00","new":3,"old":4,"arena":0,"pirate":2},{"t":"2023-05-06T19:16:49+00:00","new":4,"old":5,"arena":2,"pirate":3},{"t":"2023-05-06T19:21:01+00:00","new":6,"old":5,"arena":3,"pirate":3}],"pirates":[[6,11,4,3],[14,15,2,9],[10,16,18,20],[1,12,13,5],[8,19,17,7]],"winners":[3,2,3,2,2],"timestamp":"2023-05-06T23:14:20+00:00","lastChange":"2023-05-06T19:21:01+00:00","currentOdds":[[1,11,3,2,3],[1,13,2,7,13],[1,13,2,4,2],[1,2,10,6,6],[1,13,4,2,4]],"openingOdds":[[1,11,3,2,4],[1,13,2,5,13],[1,13,2,5,2],[1,2,8,5,5],[1,13,3,2,4]]}
    "#;
    // let data = r#"
    // {"currentOdds":[[1,5,13,13,2],[1,2,13,5,13],[1,6,2,7,2],[1,2,6,13,2],[1,2,13,13,2]],"foods":[[1,7,10,22,33,14,19,24,11,12],[4,22,27,36,23,39,37,13,15,8],[16,13,14,38,20,5,9,31,18,34],[21,26,27,37,18,10,6,35,36,12],[26,18,21,17,31,37,6,9,23,35]],"lastChange":"2024-01-29T00:24:33+00:00","openingOdds":[[1,4,13,12,2],[1,2,10,4,13],[1,5,3,7,2],[1,3,4,13,2],[1,2,13,13,2]],"pirates":[[4,11,14,16],[19,10,8,9],[20,1,12,7],[2,6,3,17],[15,18,13,5]],"round":9033,"start":"2024-01-29T00:24:33+00:00","timestamp":"2024-01-29T04:50:10+00:00","winners":[0,0,0,0,0],"changes":[{"arena":2,"new":7,"old":5,"pirate":1,"t":"2024-01-29T00:39:01+00:00"},{"arena":2,"new":2,"old":3,"pirate":2,"t":"2024-01-29T00:39:01+00:00"},{"arena":2,"new":8,"old":7,"pirate":3,"t":"2024-01-29T00:39:01+00:00"},{"arena":2,"new":3,"old":2,"pirate":4,"t":"2024-01-29T00:39:01+00:00"},{"arena":2,"new":9,"old":8,"pirate":3,"t":"2024-01-29T00:39:37+00:00"},{"arena":4,"new":3,"old":2,"pirate":4,"t":"2024-01-29T00:39:55+00:00"},{"arena":2,"new":8,"old":9,"pirate":3,"t":"2024-01-29T00:40:13+00:00"},{"arena":2,"new":9,"old":8,"pirate":3,"t":"2024-01-29T00:40:30+00:00"},{"arena":0,"new":6,"old":4,"pirate":1,"t":"2024-01-29T00:44:14+00:00"},{"arena":0,"new":13,"old":12,"pirate":3,"t":"2024-01-29T00:44:14+00:00"},{"arena":2,"new":8,"old":9,"pirate":3,"t":"2024-01-29T00:45:23+00:00"},{"arena":2,"new":9,"old":8,"pirate":3,"t":"2024-01-29T00:45:40+00:00"},{"arena":1,"new":13,"old":10,"pirate":2,"t":"2024-01-29T00:48:54+00:00"},{"arena":4,"new":2,"old":3,"pirate":4,"t":"2024-01-29T00:48:54+00:00"},{"arena":3,"new":2,"old":3,"pirate":1,"t":"2024-01-29T00:54:07+00:00"},{"arena":3,"new":6,"old":4,"pirate":2,"t":"2024-01-29T00:54:07+00:00"},{"arena":0,"new":5,"old":6,"pirate":1,"t":"2024-01-29T00:59:48+00:00"},{"arena":2,"new":2,"old":3,"pirate":4,"t":"2024-01-29T01:10:27+00:00"},{"arena":2,"new":8,"old":9,"pirate":3,"t":"2024-01-29T01:38:37+00:00"},{"arena":2,"new":6,"old":7,"pirate":1,"t":"2024-01-29T02:43:32+00:00"},{"arena":1,"new":5,"old":4,"pirate":3,"t":"2024-01-29T02:45:01+00:00"},{"arena":2,"new":7,"old":8,"pirate":3,"t":"2024-01-29T04:42:23+00:00"}]}
    // "#;

    let nfc = NeoFoodClub::from_json(data, Some(8008), None, None).unwrap();

    let bets = nfc.make_bustproof_bets().unwrap();
    // let mut bets = nfc.make_crazy_bets();
    // let bets = nfc.make_random_bets();
    // let bets = nfc.make_maxter_bets();
    // let mut bets = nfc.make_gambit_bets(0x12482);
    // let mut bets = nfc.make_random_gambit_bets();
    // let mut bets = nfc.make_gambit_bets(nfc.winners_binary());
    // let mut bets = nfc.make_winning_gambit_bets().unwrap();
    // let mut bets = nfc.make_best_gambit_bets();

    // bets.fill_bet_amounts();

    println!("{:?}", nfc.make_url(&bets));
    println!("{:?}", nfc.get_win_units(&bets));
    println!("{:?}", nfc.get_win_np(&bets));
    // println!("{:?}", nfc.winning_pirates());
    // println!("{:?}", bets.is_crazy());

    // println!("{:?}", bets.bets_hash());

    // println!("All bets: {:#?}", bets);

    // let bets = Bets::from_binaries(nfc, vec![0x1]);
    // println!("Bets: {:#?}", bets);
    // println!("bustproof: {:#?}", bets.is_bustproof());
    // println!("NeoFoodClub: {:?}", nfc);
    // let pirates = nfc.arenas.get_pirates_from_binary(0x10000);
    // println!("Pirates: {:?}", nfc.winners_pirates());
}
