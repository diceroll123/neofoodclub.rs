use std::f64::consts::E;

use crate::arena::Arenas;

#[derive(Debug, Clone)]
pub struct MultinomialLogitModel {}

impl MultinomialLogitModel {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(arenas: &Arenas) -> [[f64; 5]; 5] {
        make_probabilities(arenas)
    }
}

pub fn make_probabilities(arenas: &Arenas) -> [[f64; 5]; 5] {
    let mut probs = [[1.0, 0.0, 0.0, 0.0, 0.0]; 5];

    for arena in &arenas.arenas {
        let mut capabilities = [0.0; 5];
        for pirate in &arena.pirates {
            let pirate_index = pirate.index - 1;
            let mut pirate_strength = LOGIT_INTERCEPTS[pirate.id as usize - 1];
            let favorite = pirate.pfa.unwrap_or(0);
            let allergy = pirate.nfa.unwrap_or(0);
            pirate_strength += LOGIT_PFA[pirate.id as usize - 1] * favorite as f64;
            pirate_strength += LOGIT_NFA[pirate.id as usize - 1] * allergy as f64;

            match pirate_index {
                1 => pirate_strength += LOGIT_IS_POS2[pirate.id as usize - 1],
                2 => pirate_strength += LOGIT_IS_POS3[pirate.id as usize - 1],
                3 => pirate_strength += LOGIT_IS_POS4[pirate.id as usize - 1],
                _ => (),
            }

            capabilities[pirate_index as usize + 1] = E.powf(pirate_strength);
            capabilities[0] += capabilities[pirate_index as usize + 1];
        }

        for pirate in &arena.pirates {
            probs[arena.id as usize][pirate.index as usize] =
                capabilities[pirate.index as usize] / capabilities[0];
        }
    }

    probs
}

// these magic numbers are to be updated from time to time, surely.
// for more info: https://github.com/arsdragonfly/neofoodclub

static LOGIT_INTERCEPTS: [f64; 20] = [
    -0.5505653467394124,
    -2.3848388387111976,
    -3.478558254027841,
    -1.3890053586244873,
    -1.9176565939575803,
    -2.5675152075793033,
    -2.3143353273249554,
    -2.8313558799919383,
    -3.9019335823968233,
    -3.5882258128035347,
    -3.148241571143587,
    -2.169326502336402,
    -1.7062936735036478,
    -2.5503454346078662,
    0.0,
    -1.2578784592762349,
    -1.059757385133957,
    -2.1826351058662317,
    -0.5605783719468618,
    -1.6608180038196982,
];

static LOGIT_PFA: [f64; 20] = [
    0.15751645987509694,
    0.26306055273281875,
    0.2510034096704227,
    0.15957937973235922,
    0.2765431062703744,
    0.31686653297964323,
    0.24768920967758712,
    0.285786215512296,
    0.41136162216849836,
    0.19728776166082862,
    0.1734956834280819,
    0.1990091706829303,
    0.21651930132706249,
    0.24635467349368864,
    0.2830290762546854,
    0.18232531437739224,
    0.16134106567663997,
    0.17818977312520964,
    0.22463869805679468,
    0.263746530591703,
];

static LOGIT_NFA: [f64; 20] = [
    0.4848181644060171,
    0.29222662204607447,
    0.3081939124010599,
    0.5563766549979002,
    0.3769723616138682,
    0.40991670899985494,
    0.27537280651947094,
    0.30379969759393904,
    0.23787936378849991,
    0.36415617245862325,
    0.39280999692152224,
    0.4926557869840621,
    0.47491197095698306,
    0.3458679227200068,
    0.5148615215428655,
    0.4190387704162794,
    0.467664111731556,
    0.47126361294532254,
    0.39898657940724974,
    0.3496888311601071,
];

static LOGIT_IS_POS2: [f64; 20] = [
    0.021158502802025428,
    0.03925417444943404,
    0.26431710202585473,
    0.31204429700932157,
    0.2958881513832007,
    0.35684570379893654,
    0.29791053710022725,
    -0.11960842734248468,
    0.14139644699383916,
    0.5322022445170629,
    0.5803122626887958,
    0.1789614080028699,
    0.35757006302708166,
    0.17338557991857295,
    0.09614330673313873,
    0.04440766774743298,
    0.005601266028481538,
    0.3639425702087654,
    0.2017361653921105,
    0.22341637538608014,
];

static LOGIT_IS_POS3: [f64; 20] = [
    0.2939627190206121,
    0.4130356702811393,
    0.6063865575638252,
    0.552110704899289,
    0.6067388559201926,
    0.535076605287585,
    0.6017889410092438,
    0.09687539841588022,
    0.5246865975316289,
    0.955721909292628,
    0.638887704243457,
    0.5345584917407379,
    0.6023746907980592,
    0.4677057109696638,
    0.41924324400559815,
    0.3342400003455908,
    0.1814355382118914,
    0.5712980298733475,
    0.5188904607014326,
    0.6170900411945157,
];

static LOGIT_IS_POS4: [f64; 20] = [
    0.47071198282107324,
    0.6068520106680823,
    0.8057835563581863,
    0.8603270179693671,
    0.8307141863013495,
    0.7744623469044476,
    0.7588736594904442,
    0.537381718645823,
    0.8503685148423967,
    1.0968633716245804,
    1.021466842781995,
    0.9041512251652759,
    0.9873876941901989,
    0.7178740178709884,
    0.542178134331314,
    0.6754851261575676,
    0.5015137805345499,
    0.8849107940325963,
    0.7538567262883,
    0.9079073224460276,
];
