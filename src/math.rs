use std::collections::{BTreeMap, HashMap};

use rand::Rng;

use crate::chance::Chance;
use std::sync::OnceLock;

pub const BET_AMOUNT_MIN: u32 = 1;
pub const BET_AMOUNT_MAX: u32 = 70304;

// WARNING: the literal integers in this file switches between hex and binary willy-nilly, mostly for readability.

// each arena, as if they were full. this is impossible to actually do.
// BIT_MASKS[i] will accept pirates from arena i and only them. BIT_MASKS[4] == 0b1111, BIT_MASKS[3] == 0b11110000, etc...
pub const BIT_MASKS: [u32; 5] = [0xF0000, 0xF000, 0xF00, 0xF0, 0xF];

// represents each arena with the same pirate index filled.
// PIR_IB[i] will accept pirates of index i (from 0 to 3) PIR_IB[0] = 0b10001000100010001000, PIR_IB[1] = 0b01000100010001000100, PIR_IB[2] = 0b00100010001000100010, PIR_IB[3] = 0b00010001000100010001
// 0x88888 = (1, 1, 1, 1, 1), which is the first pirate in each arena, and so on.
const PIR_IB: [u32; 4] = [0x88888, 0x44444, 0x22222, 0x11111];

// 0xFFFFF = 0b11111111111111111111 (20 '1's), will accept all pirates
const CONVERT_PIR_IB: [u32; 5] = [0xFFFFF, 0x88888, 0x44444, 0x22222, 0x11111];

static VALID_AMOUNT_HASH_REGEX: OnceLock<regex::Regex> = OnceLock::new();
static VALID_BETS_HASH_REGEX: OnceLock<regex::Regex> = OnceLock::new();

/// ```
/// let bin = neofoodclub::math::pirate_binary(3, 2);
/// assert_eq!(bin, 0x200);
/// ```
#[inline]
pub fn pirate_binary(index: u8, arena: u8) -> u32 {
    match index {
        1..=4 => 0x80000 >> ((index - 1) + arena * 4),
        _ => 0,
    }
}

/// ```
/// let bin = neofoodclub::math::pirates_binary([0, 1, 2, 3, 4]);
/// assert_eq!(bin, 0x08421);
/// ```
#[inline]
pub fn pirates_binary(bets_indices: [u8; 5]) -> u32 {
    bets_indices
        .iter()
        .enumerate()
        .fold(0, |total, (arena, index)| {
            total | pirate_binary(*index, arena as u8)
        })
}

/// ```
/// let bin = neofoodclub::math::random_full_pirates_binary();
/// assert_eq!(bin.count_ones(), 5);
/// ```
#[inline]
pub fn random_full_pirates_binary() -> u32 {
    let mut rng = rand::rng();

    pirates_binary([
        rng.random_range(1..=4),
        rng.random_range(1..=4),
        rng.random_range(1..=4),
        rng.random_range(1..=4),
        rng.random_range(1..=4),
    ])
}

/// ```
/// let indices = neofoodclub::math::binary_to_indices(1);
/// assert_eq!(indices, [0, 0, 0, 0, 4]);
/// ```
#[inline]
pub fn binary_to_indices(binary: u32) -> [u8; 5] {
    let mut indices = [0; 5];

    for (i, index) in indices.iter_mut().enumerate() {
        let nibble = (binary >> (4 * (4 - i))) & 0xF;

        if nibble != 0 {
            *index = 4 - nibble.trailing_zeros() as u8;
        }
    }
    indices
}

#[inline]
pub fn bets_hash_regex_check(bets_hash: &str) -> Result<(), String> {
    let valid_bets_hash_regex =
        VALID_BETS_HASH_REGEX.get_or_init(|| regex::Regex::new("^[a-y]*$").unwrap());

    if !valid_bets_hash_regex.is_match(bets_hash) {
        return Err(format!(
            "Invalid bet hash '{}'. Must contain only characters a-y.",
            bets_hash
        ));
    }
    Ok(())
}

/// Returns the bet indices from a given bet hash.
/// ```
/// let bin = neofoodclub::math::bets_hash_to_bet_indices("").unwrap();
/// assert_eq!(bin, Vec::<[u8;5]>::new());
///
/// let bin = neofoodclub::math::bets_hash_to_bet_indices("f").unwrap();
/// assert_eq!(bin, [[1, 0, 0, 0, 0]]);
///
/// let bin = neofoodclub::math::bets_hash_to_bet_indices("faa").unwrap();
/// assert_eq!(bin, [[1, 0, 0, 0, 0]]);
///
/// let bin = neofoodclub::math::bets_hash_to_bet_indices("faafaafaafaafaafaa").unwrap();
/// assert_eq!(bin, [[1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 1, 0, 0], [0, 0, 0, 1, 0], [0, 0, 0, 0, 1], [1, 0, 0, 0, 0]]);
///
/// let bin = neofoodclub::math::bets_hash_to_bet_indices("jmbcoemycobmbhofmdcoamyck").unwrap();
/// assert_eq!(bin, [[1, 4, 2, 2, 0], [1, 0, 2, 2, 4], [0, 4, 2, 2, 4], [4, 0, 2, 2, 4], [0, 1, 2, 2, 0], [1, 1, 2, 2, 4], [1, 0, 2, 2, 0], [3, 0, 2, 2, 4], [0, 0, 2, 2, 4], [4, 0, 2, 2, 0]]);
/// ```
#[inline]
pub fn bets_hash_to_bet_indices(bets_hash: &str) -> Result<Vec<[u8; 5]>, String> {
    bets_hash_regex_check(bets_hash)?;

    let indices: Vec<u8> = bets_hash.bytes().map(|byte| byte - b'a').collect();

    let mut output: Vec<u8> = indices
        .iter()
        .flat_map(|&e| vec![(e as f64 / 5.0).floor() as u8, (e % 5)])
        .collect();

    // make sure the length is a multiple of 5
    let difference = output.len() % 5;
    if difference != 0 {
        output.resize(output.len() + (5 - difference), 0);
    }

    // due to the way this algorithm works, there could be resulting chunks that are entirely all 0,
    // so we filter them out.
    // good examples:
    // "faa" -> [[1, 0, 0, 0, 0,], [0]]
    // "faafaafaafaafaafaa" -> [[1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 1, 0, 0], [0, 0, 0, 1, 0], [0, 0, 0, 0, 1], [0, 0, 0, 0, 0], [1, 0, 0, 0, 0]]
    // --------------------------------------------------------------------------------------------------------------^ note the array containing all zeros

    Ok(output
        .chunks(5)
        .filter_map(|chunk| {
            if chunk.iter().any(|&n| n > 0) {
                Some(chunk.try_into().unwrap())
            } else {
                None
            }
        })
        .collect())
}

/// Returns the amount of bets from a given bet hash.
/// ```
/// let count = neofoodclub::math::bets_hash_to_bets_count("faa").unwrap();
/// assert_eq!(count, 1);
///
/// let count = neofoodclub::math::bets_hash_to_bets_count("faafaafaafaafaafaa").unwrap();
/// assert_eq!(count, 6);
///
/// let count = neofoodclub::math::bets_hash_to_bets_count("jmbcoemycobmbhofmdcoamyck").unwrap();
/// assert_eq!(count, 10);
///
/// let count = neofoodclub::math::bets_hash_to_bets_count("dgpqsxgtqsigqqsngrqsegpvsdgfqqsgsqsdgk").unwrap();
/// assert_eq!(count, 15);
/// ```
#[inline]
pub fn bets_hash_to_bets_count(bets_hash: &str) -> Result<usize, String> {
    bets_hash_regex_check(bets_hash)?;
    Ok(bets_hash_to_bet_indices(bets_hash)?.len())
}

/// Returns the hash of the given bet amounts.
/// ```
/// let hash = neofoodclub::math::bet_amounts_to_amounts_hash(&vec![Some(50), Some(100), Some(150), Some(200), Some(250)]);
/// assert_eq!(hash, "AaYAbWAcUAdSAeQ");
///
/// let hash = neofoodclub::math::bet_amounts_to_amounts_hash(&vec![None, Some(50), Some(100), Some(150), Some(200), Some(250)]);
/// assert_eq!(hash, "AaaAaYAbWAcUAdSAeQ");
///
/// let hash = neofoodclub::math::bet_amounts_to_amounts_hash(&vec![None, None, None, None, None, None, None, None, None, None]);
/// assert_eq!(hash, "AaaAaaAaaAaaAaaAaaAaaAaaAaaAaa");
/// ```
#[inline]
pub fn bet_amounts_to_amounts_hash(bet_amounts: &[Option<u32>]) -> String {
    let mut result = vec!['\0'; bet_amounts.len() * 3];
    let mut index = result.len();

    for &value in bet_amounts.iter().rev() {
        let mut state = value.unwrap_or(0) % BET_AMOUNT_MAX + BET_AMOUNT_MAX;

        for _ in 0..3 {
            index -= 1;
            let letter_index = (state % 52) as u8;
            state /= 52;

            result[index] = if letter_index < 26 {
                (letter_index + b'a') as char
            } else {
                (letter_index + b'A' - 26) as char
            };
        }
    }

    result.iter().collect()
}

/// Returns the bet amounts from a given bet amounts hash.
/// Each element in the resulting vector is an Option, where None means that the bet amount is invalid.
/// "Invalid" here means below 1.
/// ```
/// let amounts = neofoodclub::math::amounts_hash_to_bet_amounts("AaYAbWAcUAdSAeQ").unwrap();
/// assert_eq!(amounts, vec![Some(50), Some(100), Some(150), Some(200), Some(250)]);
/// let amounts = neofoodclub::math::amounts_hash_to_bet_amounts("EmxCoKCoKCglDKUCYqEXkByWBpqzGO").unwrap();
/// assert_eq!(amounts, vec![Some(11463), Some(6172), Some(6172), Some(5731), Some(10030), Some(8024), Some(13374), Some(4000), Some(3500), None]);
/// ```
#[inline]
pub fn amounts_hash_to_bet_amounts(amounts_hash: &str) -> Result<Vec<Option<u32>>, String> {
    // check that the hash matches regex "^[a-zA-Z]*$" using regex
    let valid_hash_regex =
        VALID_AMOUNT_HASH_REGEX.get_or_init(|| regex::Regex::new("^[a-zA-Z]*$").unwrap());

    // Check that the hash matches the regex
    if !valid_hash_regex.is_match(amounts_hash) {
        return Err(format!(
            "Invalid amounts hash '{}'. Must contain only characters a-z and A-Z.",
            amounts_hash
        ));
    }

    Ok(amounts_hash
        .as_bytes()
        .chunks(3)
        .map(|chunk| {
            let mut value = 0_u32;

            for &byte in chunk {
                value *= 52;
                let index = if let b'a'..=b'z' = byte {
                    (byte - b'a') as u32
                } else {
                    (byte - b'A' + 26) as u32
                };
                value += index;
            }

            let value = value.saturating_sub(BET_AMOUNT_MAX);
            Some(value).filter(|&v| v >= BET_AMOUNT_MIN)
        })
        .collect())
}

/// Returns the bet binaries from a given bet hash.
/// ```
/// let bins = neofoodclub::math::bets_hash_to_bet_binaries("faa").unwrap();
/// assert_eq!(bins, vec![0x80000]);
///
/// let bins = neofoodclub::math::bets_hash_to_bet_binaries("faafaafaafaafaafaa").unwrap();
/// assert_eq!(bins, vec![0x80000, 0x8000, 0x800, 0x80, 0x8, 0x80000]);
///
/// let bins = neofoodclub::math::bets_hash_to_bet_binaries("ltqvqwgimhqtvrnywrwvijwnn").unwrap();
/// assert_eq!(bins, vec![0x48212, 0x81828, 0x14888, 0x24484, 0x28211, 0x82442, 0x11142, 0x41418, 0x82811, 0x44242]);
///```
#[inline]
pub fn bets_hash_to_bet_binaries(bets_hash: &str) -> Result<Vec<u32>, String> {
    bets_hash_regex_check(bets_hash)?;
    Ok(bets_hash_to_bet_indices(bets_hash)?
        .iter()
        .map(|&indices| pirates_binary(indices))
        .collect())
}

/// Returns the hash value from a given bet indices.
/// ```
/// let hash = neofoodclub::math::bets_hash_value(vec![[1, 0, 0, 0, 0]]);
/// assert_eq!(hash, "faa");
/// ```
#[inline]
pub fn bets_hash_value(bets_indices: Vec<[u8; 5]>) -> String {
    let len = bets_indices.len();

    bets_indices
        .into_iter()
        .flatten()
        .chain(std::iter::once(0).take(len & 1))
        .collect::<Vec<u8>>()
        .chunks_exact(2)
        .map(|chunk| {
            // char_index is the index of the character in the alphabet
            // 0 = a, 1 = b, 2 = c, ..., 25 = z
            let char_index = chunk[0] * 5 + chunk[1];
            // b'a' is the byte literal for the ASCII "a", which is 97
            (b'a' + char_index) as char
        })
        .collect()
}

/// Returns the bet binaries from bet indices.
/// ```
/// let bins = neofoodclub::math::bets_indices_to_bet_binaries(vec![[1, 0, 0, 0, 0]]);
/// assert_eq!(bins, vec![0x80000]);
///
/// let bins = neofoodclub::math::bets_indices_to_bet_binaries(vec![[1, 0, 0, 0, 0], [0, 1, 0, 0, 0], [0, 0, 1, 0, 0], [0, 0, 0, 1, 0], [0, 0, 0, 0, 1], [1, 0, 0, 0, 0]]);
/// assert_eq!(bins, vec![0x80000, 0x8000, 0x800, 0x80, 0x8, 0x80000]);
/// ```
#[inline]
pub fn bets_indices_to_bet_binaries(bets_indices: Vec<[u8; 5]>) -> Vec<u32> {
    bets_indices
        .iter()
        .map(|&indices| pirates_binary(indices))
        .collect()
}

#[inline]
fn ib_doable(binary: u32) -> bool {
    BIT_MASKS.iter().all(|&mask| binary & mask != 0)
}

#[inline]
fn ib_prob(binary: u32, probabilities: [[f64; 5]; 5]) -> f64 {
    // computes the probability that the winning combination is accepted by ib
    BIT_MASKS
        .iter()
        .enumerate()
        .fold(1.0, |total_prob, (x, bit_mask)| {
            let ar_prob: f64 = PIR_IB
                .iter()
                .enumerate()
                .map(|(y, &pir_ib)| {
                    if binary & bit_mask & pir_ib > 0 {
                        probabilities[x][y + 1]
                    } else {
                        0.0
                    }
                })
                .sum();
            total_prob * ar_prob
        })
}

pub fn expand_ib_object(bets: &[[u8; 5]], bet_odds: &[u32]) -> HashMap<u32, u32> {
    // makes a dict of permutations of the pirates + odds
    // this is why the bet table could be very long

    let mut bets_to_ib: HashMap<u32, u32> = HashMap::new();
    for (key, bet_value) in bets.iter().enumerate() {
        let ib = bet_value
            .iter()
            .zip(BIT_MASKS.iter())
            .fold(0, |acc, (&v, &m)| acc | CONVERT_PIR_IB[v as usize] & m);
        *bets_to_ib.entry(ib).or_insert(0) += bet_odds[key];
    }

    // filters down the doable bets from the permutations above
    let mut res: HashMap<u32, u32> = HashMap::new();
    res.insert(0xFFFFF, 0);
    let mut bets_to_ib: Vec<_> = bets_to_ib.into_iter().collect();
    bets_to_ib.sort();
    for (ib_bet, winnings) in bets_to_ib.into_iter() {
        let drained_elements: Vec<_> = res
            .keys()
            .copied()
            .filter(|ib_key| ib_doable(ib_bet & ib_key))
            .collect();
        for mut ib_key in drained_elements.into_iter() {
            let com = ib_bet & ib_key;
            let val_key = res
                .remove(&ib_key)
                .expect("Failed to retrieve value for ib_key");
            res.insert(com, winnings + val_key);
            for ar in BIT_MASKS {
                let tst = ib_key ^ (com & ar);
                if !ib_doable(tst) {
                    continue;
                }
                res.insert(tst, val_key);
                ib_key = (ib_key & !ar) | (com & ar);
            }
        }
    }
    res
}

#[derive(Debug, Clone)]
pub struct RoundDictData {
    pub bins: Vec<u32>,
    pub probs: Vec<f64>,
    pub odds: Vec<u32>,
    pub ers: Vec<f64>,
    pub maxbets: Vec<u32>,
}

pub fn make_round_dicts(stds: [[f64; 5]; 5], odds: [[u8; 5]; 5]) -> RoundDictData {
    let mut bins: Vec<u32> = Vec::with_capacity(3124);
    let mut probs: Vec<f64> = Vec::with_capacity(3124);
    let mut odds_vec: Vec<u32> = Vec::with_capacity(3124);
    let mut ers: Vec<f64> = Vec::with_capacity(3124);
    let mut maxbets: Vec<u32> = Vec::with_capacity(3124);

    for a in 0..5 {
        for b in 0..5 {
            for c in 0..5 {
                for d in 0..5 {
                    for e in 0..5 {
                        if a == 0 && b == 0 && c == 0 && d == 0 && e == 0 {
                            continue;
                        }

                        let nums = [a, b, c, d, e];
                        let total_bin: u32 = pirates_binary(nums);

                        let (total_probs, total_odds) = nums.iter().enumerate().fold(
                            (1.0, 1),
                            |(probs, odds_fold), (arena, &index)| {
                                if index == 0 {
                                    (probs, odds_fold)
                                } else {
                                    (
                                        probs * stds[arena][index as usize],
                                        odds_fold * odds[arena][index as usize] as u32,
                                    )
                                }
                            },
                        );

                        let er = total_probs * total_odds as f64;
                        let maxbet = (1_000_000.0 / total_odds as f64).ceil() as u32;

                        bins.push(total_bin);
                        probs.push(total_probs);
                        odds_vec.push(total_odds);
                        ers.push(er);
                        maxbets.push(maxbet);
                    }
                }
            }
        }
    }

    RoundDictData {
        bins,
        probs,
        odds: odds_vec,
        ers,
        maxbets,
    }
}

pub fn build_chance_objects(
    bets: &[[u8; 5]],
    bet_odds: &[u32],
    probabilities: [[f64; 5]; 5],
) -> Vec<Chance> {
    let expanded = expand_ib_object(bets, bet_odds);
    let mut win_table: BTreeMap<u32, f64> = BTreeMap::new();
    for (key, value) in expanded.iter() {
        *win_table.entry(*value).or_insert(0.0) += ib_prob(*key, probabilities);
    }

    let mut cumulative: f64 = 0.0;
    let mut tail: f64 = 1.0;
    let mut chances: Vec<Chance> = Vec::with_capacity(win_table.len());
    for (key, value) in win_table.into_iter() {
        cumulative += value;
        chances.push(Chance {
            value: key,
            probability: value,
            cumulative,
            tail,
        });

        tail -= value;
    }
    chances
}
