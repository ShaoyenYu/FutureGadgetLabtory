//! Random-policy simulation, for stress tests and as a placeholder seat policy.

use rand::seq::{IteratorRandom, SliceRandom};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::config::GameConfig;
use crate::decision::{Decision, DecisionRequest, Direction, WitchAction};
use crate::event::{NightStep, Step};
use crate::game::Game;
use crate::role::Role;
use crate::seat::SeatId;

const MAX_ACTIONS: usize = 20_000;

#[derive(Debug, Error)]
pub enum SimError {
    #[error("game did not finish within {0} actions")]
    Stalled(usize),
}

fn pick<R: Rng + ?Sized>(candidates: &[SeatId], rng: &mut R, none_p: f64) -> Option<SeatId> {
    if candidates.is_empty() || rng.gen_bool(none_p) {
        None
    } else {
        candidates.choose(rng).copied()
    }
}

/// A uniformly random valid answer to `req`.
pub fn random_decision<R: Rng + ?Sized>(req: &DecisionRequest, rng: &mut R) -> Decision {
    match req {
        DecisionRequest::WolfKill { candidates, .. } => Decision::WolfKill(pick(candidates, rng, 0.05)),
        DecisionRequest::WitchAct { can_save, can_poison, poison_candidates, .. } => {
            let r: f64 = rng.gen();
            if *can_save && r < 0.5 {
                Decision::Witch(WitchAction::Save)
            } else if *can_poison && r < 0.7 && !poison_candidates.is_empty() {
                Decision::Witch(WitchAction::Poison(*poison_candidates.choose(rng).unwrap()))
            } else {
                Decision::Witch(WitchAction::Pass)
            }
        }
        DecisionRequest::SeerCheck { candidates } => Decision::SeerCheck(pick(candidates, rng, 0.0)),
        DecisionRequest::SheriffSignup => Decision::SheriffSignup(rng.gen_bool(0.4)),
        DecisionRequest::SheriffWithdrawWindow => {
            if rng.gen_bool(0.15) {
                Decision::SheriffWithdraw
            } else {
                Decision::Stay
            }
        }
        DecisionRequest::Speak { kind } => Decision::Speak(format!("{kind:?}")),
        DecisionRequest::Vote { candidates, .. } => Decision::Vote(pick(candidates, rng, 0.1)),
        DecisionRequest::HunterShoot { candidates } => Decision::HunterShoot(pick(candidates, rng, 0.2)),
        DecisionRequest::BadgeTransfer { candidates } => Decision::BadgeTransfer(pick(candidates, rng, 0.2)),
        DecisionRequest::ChooseSpeakOrder => Decision::SpeakOrder(if rng.gen_bool(0.5) {
            Direction::Clockwise
        } else {
            Direction::CounterClockwise
        }),
    }
}

/// Plays a whole game with random seats, including out-of-band wolf chat,
/// mid-speech withdrawals and timeouts.
pub fn simulate(cfg: GameConfig, seed: u64) -> Result<Game, SimError> {
    let mut game = Game::new(cfg, seed);
    let mut rng = ChaCha8Rng::seed_from_u64(seed ^ 0x5eed_5eed);
    for _ in 0..MAX_ACTIONS {
        if game.is_over() {
            return Ok(game);
        }
        match game.step() {
            Step::Night(NightStep::WolfKill) if rng.gen_bool(0.2) => {
                let wolf = game
                    .alive_seats()
                    .into_iter()
                    .filter(|s| game.role_of(*s) == Role::Werewolf)
                    .choose(&mut rng);
                if let Some(w) = wolf {
                    game.submit(w, Decision::WolfChat("刀谁?".into())).expect("wolf chat is valid");
                }
                continue;
            }
            Step::SheriffSpeech if rng.gen_bool(0.05) => {
                if let Some(&c) = game.sheriff_candidates().choose(&mut rng) {
                    game.submit(c, Decision::SheriffWithdraw).expect("withdraw is valid");
                }
                continue;
            }
            _ => {}
        }
        if rng.gen_bool(0.03) {
            game.timeout();
            continue;
        }
        let pending: Vec<(SeatId, DecisionRequest)> =
            game.pending().iter().map(|(s, r)| (*s, r.clone())).collect();
        let (seat, req) = pending.choose(&mut rng).expect("a running game always awaits someone");
        let d = random_decision(req, &mut rng);
        game.submit(*seat, d).expect("random decisions are valid");
    }
    Err(SimError::Stalled(MAX_ACTIONS))
}
