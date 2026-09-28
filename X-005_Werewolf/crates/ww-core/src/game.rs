use std::collections::{BTreeMap, VecDeque};

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use thiserror::Error;

use crate::config::{ConfigError, GameConfig, SelfSave, WinRule};
use crate::decision::{Decision, DecisionRequest, Direction, SpeechKind, VoteKind, WitchAction};
use crate::event::{DeathCause, Event, EventKind, NightStep, Phase, Step, Visibility};
use crate::role::{Category, Faction, Role};
use crate::seat::SeatId;
use crate::visibility::Observation;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RuleError {
    #[error("the game is over")]
    GameOver,
    #[error("unknown seat {0}")]
    UnknownSeat(SeatId),
    #[error("{0} has no pending decision")]
    NotPending(SeatId),
    #[error("invalid decision from {seat}: {reason}")]
    InvalidDecision { seat: SeatId, reason: String },
}

#[derive(Debug, Clone)]
struct DeathTask {
    seat: SeatId,
    cause: DeathCause,
    last_words: bool,
}

/// Per-death sub-steps, in the order they are asked: 遗言 → 开枪 → 移交警徽.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeathStage {
    LastWords,
    Hunter,
    Badge,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AfterDeaths {
    SpeakOrder,
    Night,
}

/// The authoritative game state (god view).
///
/// Drive it by answering [`Game::pending`] requests with [`Game::submit`], or let
/// the clock run out with [`Game::timeout`]. Both return the new events.
/// Concurrent steps (night skills, votes, sign-ups) resolve once every pending seat
/// has answered; speech steps ask one seat at a time.
#[derive(Debug, Clone)]
pub struct Game {
    cfg: GameConfig,
    rng: ChaCha8Rng,
    roles: Vec<Role>,
    alive: Vec<bool>,
    round: u32,
    phase: Phase,
    step: Step,
    events: Vec<Event>,
    pending: BTreeMap<SeatId, DecisionRequest>,
    submitted: BTreeMap<SeatId, Decision>,

    wolf_votes: Vec<(SeatId, Option<SeatId>)>,
    night_kill: Option<SeatId>,
    night_saved: bool,
    night_poison: Option<SeatId>,
    witch_has_save: bool,
    witch_has_poison: bool,
    /// Night deaths are only applied when announced, so the night's victims can
    /// still run for and vote in the sheriff election.
    pending_deaths: Vec<(SeatId, DeathCause)>,
    last_night_deaths: Vec<SeatId>,

    sheriff: Option<SeatId>,
    sheriff_done: bool,
    candidates: Vec<SeatId>,
    active_candidates: Vec<SeatId>,
    ballot: Vec<SeatId>,
    voters: Vec<SeatId>,

    speak_queue: VecDeque<(SeatId, SpeechKind)>,
    death_tasks: VecDeque<DeathTask>,
    death_stage: DeathStage,
    after_deaths: AfterDeaths,
    winner: Option<Faction>,
}

impl Game {
    /// Shuffles the deck with `seed` and starts the first night.
    pub fn new(cfg: GameConfig, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut roles = cfg.roles.clone();
        roles.shuffle(&mut rng);
        Self::build(cfg, roles, rng)
    }

    /// Starts a game with a fixed seat layout (tests, replays).
    pub fn with_roles(cfg: GameConfig, roles: Vec<Role>, seed: u64) -> Result<Self, ConfigError> {
        cfg.validate()?;
        let (mut a, mut b) = (roles.clone(), cfg.roles.clone());
        a.sort();
        b.sort();
        if a != b {
            return Err(ConfigError::RoleMismatch);
        }
        Ok(Self::build(cfg, roles, ChaCha8Rng::seed_from_u64(seed)))
    }

    fn build(cfg: GameConfig, roles: Vec<Role>, rng: ChaCha8Rng) -> Self {
        let n = roles.len();
        let mut g = Game {
            cfg,
            rng,
            roles,
            alive: vec![true; n],
            round: 0,
            phase: Phase::Setup,
            step: Step::Setup,
            events: Vec::new(),
            pending: BTreeMap::new(),
            submitted: BTreeMap::new(),
            wolf_votes: Vec::new(),
            night_kill: None,
            night_saved: false,
            night_poison: None,
            witch_has_save: true,
            witch_has_poison: true,
            pending_deaths: Vec::new(),
            last_night_deaths: Vec::new(),
            sheriff: None,
            sheriff_done: false,
            candidates: Vec::new(),
            active_candidates: Vec::new(),
            ballot: Vec::new(),
            voters: Vec::new(),
            speak_queue: VecDeque::new(),
            death_tasks: VecDeque::new(),
            death_stage: DeathStage::LastWords,
            after_deaths: AfterDeaths::SpeakOrder,
            winner: None,
        };
        for i in 0..n {
            let seat = SeatId::from_index(i);
            let role = g.roles[i];
            g.emit(Visibility::Private(seat), EventKind::RoleAssigned { seat, role });
        }
        let wolves = g.seats_where(|r| r.faction() == Faction::Wolf);
        g.emit(Visibility::Team(Faction::Wolf), EventKind::WolfTeammates { wolves });
        g.enter_night();
        g.run();
        g
    }

    // ---------------------------------------------------------------- public API

    pub fn config(&self) -> &GameConfig {
        &self.cfg
    }

    pub fn players(&self) -> usize {
        self.roles.len()
    }

    /// God view. Only the moderator may call this; players learn roles via observations.
    pub fn role_of(&self, seat: SeatId) -> Role {
        self.roles[seat.index()]
    }

    pub fn is_alive(&self, seat: SeatId) -> bool {
        self.alive[seat.index()]
    }

    pub fn alive_seats(&self) -> Vec<SeatId> {
        (0..self.players()).filter(|&i| self.alive[i]).map(SeatId::from_index).collect()
    }

    pub fn round(&self) -> u32 {
        self.round
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn sheriff(&self) -> Option<SeatId> {
        self.sheriff
    }

    /// Candidates still running (not withdrawn) in the current sheriff election.
    pub fn sheriff_candidates(&self) -> &[SeatId] {
        &self.active_candidates
    }

    pub fn winner(&self) -> Option<Faction> {
        self.winner
    }

    pub fn is_over(&self) -> bool {
        self.winner.is_some()
    }

    /// Full god-view event log.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Decisions currently awaited, keyed by seat. Each request goes only to its seat.
    pub fn pending(&self) -> &BTreeMap<SeatId, DecisionRequest> {
        &self.pending
    }

    pub fn submit(&mut self, seat: SeatId, decision: Decision) -> Result<Vec<Event>, RuleError> {
        let start = self.events.len();
        self.accept(seat, decision)?;
        self.run();
        Ok(self.events[start..].to_vec())
    }

    /// Applies the default decision for every seat currently pending.
    pub fn timeout(&mut self) -> Vec<Event> {
        let start = self.events.len();
        let due: Vec<(SeatId, DecisionRequest)> =
            self.pending.iter().map(|(s, r)| (*s, r.clone())).collect();
        for (seat, req) in due {
            // A sequential step may have moved on to a new request for the same seat.
            if self.pending.get(&seat) == Some(&req) {
                self.accept(seat, req.default_decision()).expect("default decisions are always valid");
            }
        }
        self.run();
        self.events[start..].to_vec()
    }

    pub fn can_see(&self, seat: SeatId, vis: Visibility) -> bool {
        match vis {
            Visibility::Public => true,
            Visibility::Team(f) => self.role_of(seat).faction() == f,
            Visibility::Private(s) => s == seat,
            Visibility::GodOnly => false,
        }
    }

    /// The single exit point for information towards a seat.
    pub fn observe(&self, event: &Event, seat: SeatId) -> Option<Observation> {
        self.can_see(seat, event.vis).then(|| Observation::from_event(event))
    }

    pub fn observations_for(&self, seat: SeatId) -> Vec<Observation> {
        self.events.iter().filter_map(|e| self.observe(e, seat)).collect()
    }

    // ---------------------------------------------------------------- helpers

    fn seats_where(&self, f: impl Fn(Role) -> bool) -> Vec<SeatId> {
        (0..self.players()).filter(|&i| f(self.roles[i])).map(SeatId::from_index).collect()
    }

    fn alive_with(&self, role: Role) -> Option<SeatId> {
        (0..self.players()).find(|&i| self.alive[i] && self.roles[i] == role).map(SeatId::from_index)
    }

    fn board_has(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    fn emit(&mut self, vis: Visibility, kind: EventKind) {
        let seq = self.events.len() as u64;
        self.events.push(Event { seq, phase: self.phase, vis, kind });
    }

    fn begin(&mut self, step: Step) {
        self.step = step;
        self.pending.clear();
        self.submitted.clear();
        self.emit(Visibility::Public, EventKind::StepStarted { step });
    }

    /// Alive seats starting after `anchor` in `dir`, wrapping; `anchor` comes last if alive.
    fn ring_after(&self, anchor: SeatId, dir: Direction) -> Vec<SeatId> {
        let n = self.players() as isize;
        let a = anchor.index() as isize;
        let d = if dir == Direction::Clockwise { 1 } else { -1 };
        (1..=n)
            .map(|k| (a + d * k).rem_euclid(n) as usize)
            .filter(|&i| self.alive[i])
            .map(SeatId::from_index)
            .collect()
    }

    fn run(&mut self) {
        let mut guard = 0;
        while self.pending.is_empty() && self.winner.is_none() {
            self.finish_step();
            guard += 1;
            assert!(guard < 10_000, "state machine stalled at {:?}", self.step);
        }
    }

    // ---------------------------------------------------------------- input

    fn accept(&mut self, seat: SeatId, d: Decision) -> Result<(), RuleError> {
        if self.winner.is_some() {
            return Err(RuleError::GameOver);
        }
        if seat.0 == 0 || seat.index() >= self.players() {
            return Err(RuleError::UnknownSeat(seat));
        }

        match (self.step, &d) {
            (Step::Night(NightStep::WolfKill), Decision::WolfChat(text)) => {
                if !(self.is_alive(seat) && self.role_of(seat) == Role::Werewolf) {
                    return Err(invalid(seat, "only living wolves can use wolf chat"));
                }
                let text = text.clone();
                self.emit(Visibility::Team(Faction::Wolf), EventKind::WolfChat { from: seat, text });
                return Ok(());
            }
            (Step::SheriffSpeech, Decision::SheriffWithdraw) => {
                if !self.active_candidates.contains(&seat) {
                    return Err(invalid(seat, "not a running sheriff candidate"));
                }
                self.withdraw(seat);
                if self.pending.remove(&seat).is_some() {
                    self.advance_speech();
                }
                return Ok(());
            }
            _ => {}
        }

        let req = self.pending.get(&seat).ok_or(RuleError::NotPending(seat))?;
        validate(seat, req, &d)?;
        let req = self.pending.remove(&seat).expect("checked above");

        match self.step {
            Step::SheriffSpeech | Step::SheriffPkSpeech | Step::Speech | Step::ExilePkSpeech => {
                let (DecisionRequest::Speak { kind }, Decision::Speak(text)) = (req, d) else {
                    unreachable!("speech steps only request speeches")
                };
                self.emit(Visibility::Public, EventKind::Speech { seat, kind, text });
                self.advance_speech();
            }
            Step::DeathResolution => {
                match d {
                    Decision::Speak(text) => {
                        let kind = SpeechKind::LastWords;
                        self.emit(Visibility::Public, EventKind::Speech { seat, kind, text });
                    }
                    Decision::HunterShoot(Some(target)) => {
                        self.alive[target.index()] = false;
                        self.emit(Visibility::Public, EventKind::HunterShot { hunter: seat, target });
                        if !self.check_win() {
                            let last_words = self.cfg.rules.shot_last_words;
                            let cause = DeathCause::HunterShot;
                            self.death_tasks.push_back(DeathTask { seat: target, cause, last_words });
                        }
                    }
                    Decision::HunterShoot(None) => {}
                    Decision::BadgeTransfer(to) => {
                        self.sheriff = to;
                        self.emit(Visibility::Public, EventKind::BadgeTransferred { from: seat, to });
                    }
                    other => unreachable!("unexpected death-resolution decision {other:?}"),
                }
                self.advance_death();
            }
            _ => {
                if let Decision::WolfKill(target) = &d {
                    self.wolf_votes.push((seat, *target));
                    let target = *target;
                    self.emit(Visibility::Team(Faction::Wolf), EventKind::WolfVoteCast { from: seat, target });
                }
                self.submitted.insert(seat, d);
            }
        }
        Ok(())
    }

    /// Called whenever the current step has nothing left to wait for.
    fn finish_step(&mut self) {
        match self.step {
            Step::Setup | Step::GameOver => unreachable!("no step to finish"),
            Step::Night(NightStep::WolfKill) => {
                self.resolve_wolf_kill();
                self.enter_witch();
            }
            Step::Night(NightStep::Witch) => {
                self.resolve_witch();
                self.enter_seer();
            }
            Step::Night(NightStep::Seer) => {
                self.resolve_seer();
                self.end_night();
            }
            Step::SheriffSignup => self.resolve_sheriff_signup(),
            Step::SheriffSpeech => self.enter_sheriff_withdraw(),
            Step::SheriffWithdraw => self.resolve_sheriff_withdraw(),
            Step::SheriffVote | Step::SheriffPkVote => self.resolve_sheriff_vote(),
            Step::SheriffPkSpeech => self.enter_vote(Step::SheriffPkVote, VoteKind::SheriffPk),
            Step::Announce | Step::DeathResolution => self.continue_deaths(),
            Step::SpeakOrder => self.resolve_speak_order(),
            Step::Speech => {
                self.voters = self.alive_seats();
                self.ballot = self.alive_seats();
                self.enter_vote(Step::ExileVote, VoteKind::Exile);
            }
            Step::ExileVote | Step::ExilePkVote => self.resolve_exile_vote(),
            Step::ExilePkSpeech => {
                let voters: Vec<SeatId> =
                    self.alive_seats().into_iter().filter(|s| !self.ballot.contains(s)).collect();
                if voters.is_empty() {
                    self.exile(None);
                } else {
                    self.voters = voters;
                    self.enter_vote(Step::ExilePkVote, VoteKind::ExilePk);
                }
            }
        }
    }

    // ---------------------------------------------------------------- night

    fn enter_night(&mut self) {
        self.round += 1;
        self.phase = Phase::Night(self.round);
        self.night_kill = None;
        self.night_saved = false;
        self.night_poison = None;
        self.wolf_votes.clear();
        self.emit(Visibility::Public, EventKind::PhaseStarted { phase: self.phase });

        self.begin(Step::Night(NightStep::WolfKill));
        let wolves: Vec<SeatId> =
            self.alive_seats().into_iter().filter(|s| self.role_of(*s) == Role::Werewolf).collect();
        let candidates = self.alive_seats();
        for w in &wolves {
            let req = DecisionRequest::WolfKill { candidates: candidates.clone(), teammates: wolves.clone() };
            self.pending.insert(*w, req);
        }
    }

    /// Plurality wins; a tie goes to the most recently voted target. `None` (空刀) counts as a target.
    fn resolve_wolf_kill(&mut self) {
        let mut best: Option<(usize, usize, Option<SeatId>)> = None;
        for (idx, (_, target)) in self.wolf_votes.iter().enumerate() {
            let count = self.wolf_votes.iter().filter(|(_, t)| t == target).count();
            if best.is_none_or(|(c, i, _)| (count, idx) > (c, i)) {
                best = Some((count, idx, *target));
            }
        }
        self.night_kill = best.and_then(|b| b.2);
        let target = self.night_kill;
        self.emit(Visibility::Team(Faction::Wolf), EventKind::WolfTarget { target });
    }

    fn enter_witch(&mut self) {
        if !self.board_has(Role::Witch) {
            return self.enter_seer();
        }
        self.begin(Step::Night(NightStep::Witch));
        let Some(witch) = self.alive_with(Role::Witch) else { return };
        if !self.witch_has_save && !self.witch_has_poison {
            return;
        }
        let victim = if self.witch_has_save { self.night_kill } else { None };
        if self.witch_has_save {
            self.emit(Visibility::Private(witch), EventKind::WitchInfo { victim });
        }
        let self_save_ok = match self.cfg.rules.witch_self_save {
            SelfSave::Never => false,
            SelfSave::FirstNightOnly => self.round == 1,
            SelfSave::Always => true,
        };
        let can_save = victim.is_some() && (victim != Some(witch) || self_save_ok);
        let poison_candidates = if self.witch_has_poison {
            self.alive_seats().into_iter().filter(|s| *s != witch).collect()
        } else {
            Vec::new()
        };
        let req = DecisionRequest::WitchAct {
            victim,
            can_save,
            can_poison: self.witch_has_poison,
            poison_candidates,
        };
        self.pending.insert(witch, req);
    }

    fn resolve_witch(&mut self) {
        let Some(witch) = self.alive_with(Role::Witch) else { return };
        match self.submitted.get(&witch) {
            Some(Decision::Witch(WitchAction::Save)) => {
                let target = self.night_kill.expect("save is only offered with a victim");
                self.night_saved = true;
                self.witch_has_save = false;
                self.emit(Visibility::Private(witch), EventKind::WitchSaved { target });
            }
            Some(Decision::Witch(WitchAction::Poison(target))) => {
                let target = *target;
                self.night_poison = Some(target);
                self.witch_has_poison = false;
                self.emit(Visibility::Private(witch), EventKind::WitchPoisoned { target });
            }
            _ => {}
        }
    }

    fn enter_seer(&mut self) {
        if !self.board_has(Role::Seer) {
            return self.end_night();
        }
        self.begin(Step::Night(NightStep::Seer));
        let Some(seer) = self.alive_with(Role::Seer) else { return };
        let candidates = self.alive_seats().into_iter().filter(|s| *s != seer).collect();
        self.pending.insert(seer, DecisionRequest::SeerCheck { candidates });
    }

    fn resolve_seer(&mut self) {
        let Some(seer) = self.alive_with(Role::Seer) else { return };
        if let Some(Decision::SeerCheck(Some(target))) = self.submitted.get(&seer) {
            let target = *target;
            let is_wolf = self.role_of(target).faction() == Faction::Wolf;
            self.emit(Visibility::Private(seer), EventKind::SeerResult { target, is_wolf });
        }
    }

    fn end_night(&mut self) {
        let mut deaths = Vec::new();
        if let Some(k) = self.night_kill {
            if !self.night_saved && Some(k) != self.night_poison {
                deaths.push((k, DeathCause::WolfKill));
            }
        }
        if let Some(p) = self.night_poison {
            deaths.push((p, DeathCause::Poison));
        }
        deaths.sort_by_key(|d| d.0);
        self.emit(Visibility::GodOnly, EventKind::NightResolved { deaths: deaths.clone() });
        self.pending_deaths = deaths;

        self.phase = Phase::Day(self.round);
        self.emit(Visibility::Public, EventKind::PhaseStarted { phase: self.phase });
        if self.cfg.rules.sheriff && !self.sheriff_done && self.round == 1 {
            self.enter_sheriff_signup();
        } else {
            self.enter_announce();
        }
    }

    // ---------------------------------------------------------------- sheriff

    fn enter_sheriff_signup(&mut self) {
        self.begin(Step::SheriffSignup);
        for s in self.alive_seats() {
            self.pending.insert(s, DecisionRequest::SheriffSignup);
        }
    }

    fn resolve_sheriff_signup(&mut self) {
        let cands: Vec<SeatId> = self
            .submitted
            .iter()
            .filter(|(_, d)| matches!(d, Decision::SheriffSignup(true)))
            .map(|(s, _)| *s)
            .collect();
        self.emit(Visibility::Public, EventKind::SheriffCandidates { seats: cands.clone() });
        self.candidates = cands.clone();
        self.active_candidates = cands.clone();

        if cands.is_empty() || cands.len() == self.alive_seats().len() {
            return self.elect_sheriff(None);
        }
        if cands.len() == 1 {
            return self.elect_sheriff(Some(cands[0]));
        }
        let start = self.rng.gen_range(0..cands.len());
        self.speak_queue = cands[start..]
            .iter()
            .chain(&cands[..start])
            .map(|s| (*s, SpeechKind::SheriffCampaign))
            .collect();
        self.begin(Step::SheriffSpeech);
        self.advance_speech();
    }

    fn withdraw(&mut self, seat: SeatId) {
        self.active_candidates.retain(|s| *s != seat);
        self.emit(Visibility::Public, EventKind::SheriffWithdrew { seat });
    }

    fn enter_sheriff_withdraw(&mut self) {
        if self.active_candidates.len() <= 1 {
            return self.resolve_sheriff_withdraw();
        }
        self.begin(Step::SheriffWithdraw);
        for s in self.active_candidates.clone() {
            self.pending.insert(s, DecisionRequest::SheriffWithdrawWindow);
        }
    }

    fn resolve_sheriff_withdraw(&mut self) {
        let leaving: Vec<SeatId> = self
            .submitted
            .iter()
            .filter(|(_, d)| matches!(d, Decision::SheriffWithdraw))
            .map(|(s, _)| *s)
            .collect();
        for s in leaving {
            self.withdraw(s);
        }
        match self.active_candidates.len() {
            0 => self.elect_sheriff(None),
            1 => self.elect_sheriff(Some(self.active_candidates[0])),
            _ => {
                // Everyone who signed up, withdrawn or not, loses the right to vote.
                let voters: Vec<SeatId> =
                    self.alive_seats().into_iter().filter(|s| !self.candidates.contains(s)).collect();
                if voters.is_empty() {
                    return self.elect_sheriff(None);
                }
                self.voters = voters;
                self.ballot = self.active_candidates.clone();
                self.enter_vote(Step::SheriffVote, VoteKind::Sheriff);
            }
        }
    }

    fn resolve_sheriff_vote(&mut self) {
        let first_round = self.step == Step::SheriffVote;
        let kind = if first_round { VoteKind::Sheriff } else { VoteKind::SheriffPk };
        let (winner, tied) = self.tally(kind);
        if first_round && tied.len() > 1 {
            self.ballot = tied.clone();
            self.speak_queue = tied.iter().map(|s| (*s, SpeechKind::SheriffPk)).collect();
            self.begin(Step::SheriffPkSpeech);
            self.advance_speech();
        } else {
            self.elect_sheriff(winner);
        }
    }

    fn elect_sheriff(&mut self, seat: Option<SeatId>) {
        self.sheriff = seat;
        self.sheriff_done = true;
        self.emit(Visibility::Public, EventKind::SheriffElected { seat });
        self.enter_announce();
    }

    // ---------------------------------------------------------------- voting & speeches

    fn enter_vote(&mut self, step: Step, kind: VoteKind) {
        self.begin(step);
        for v in self.voters.clone() {
            let req = DecisionRequest::Vote { kind, candidates: self.ballot.clone() };
            self.pending.insert(v, req);
        }
    }

    /// Reveals all votes. Returns the single top seat, or the tied top seats.
    fn tally(&mut self, kind: VoteKind) -> (Option<SeatId>, Vec<SeatId>) {
        let mut score: BTreeMap<SeatId, u32> = BTreeMap::new();
        for voter in self.voters.clone() {
            let target = match self.submitted.get(&voter) {
                Some(Decision::Vote(t)) => *t,
                _ => None,
            };
            let sheriff_weighted = matches!(kind, VoteKind::Exile | VoteKind::ExilePk);
            let weight_x2 = if sheriff_weighted && self.sheriff == Some(voter) { 3 } else { 2 };
            self.emit(Visibility::Public, EventKind::VoteCast { kind, voter, target, weight_x2 });
            if let Some(t) = target {
                *score.entry(t).or_default() += weight_x2 as u32;
            }
        }
        let max = score.values().copied().max().unwrap_or(0);
        let top: Vec<SeatId> = score.iter().filter(|(_, v)| **v == max).map(|(s, _)| *s).collect();
        let (winner, tied) = match top.len() {
            1 => (Some(top[0]), Vec::new()),
            _ => (None, top),
        };
        self.emit(Visibility::Public, EventKind::VoteResult { kind, winner, tied: tied.clone() });
        (winner, tied)
    }

    fn advance_speech(&mut self) {
        if self.winner.is_some() {
            return;
        }
        while let Some((seat, kind)) = self.speak_queue.pop_front() {
            let eligible = match kind {
                SpeechKind::SheriffCampaign => self.active_candidates.contains(&seat),
                _ => self.is_alive(seat),
            };
            if eligible {
                self.pending.insert(seat, DecisionRequest::Speak { kind });
                return;
            }
        }
    }

    // ---------------------------------------------------------------- day

    fn enter_announce(&mut self) {
        self.begin(Step::Announce);
        let deaths = std::mem::take(&mut self.pending_deaths);
        for (s, _) in &deaths {
            self.alive[s.index()] = false;
        }
        self.last_night_deaths = deaths.iter().map(|d| d.0).collect();
        let seats = self.last_night_deaths.clone();
        self.emit(Visibility::Public, EventKind::DeathsAnnounced { seats });
        if self.check_win() {
            return;
        }
        let last_words = self.round == 1 && self.cfg.rules.first_night_last_words;
        self.death_tasks =
            deaths.into_iter().map(|(seat, cause)| DeathTask { seat, cause, last_words }).collect();
        self.after_deaths = AfterDeaths::SpeakOrder;
    }

    fn continue_deaths(&mut self) {
        if !self.death_tasks.is_empty() && self.step != Step::DeathResolution {
            self.begin(Step::DeathResolution);
            self.death_stage = DeathStage::LastWords;
            self.advance_death();
            return;
        }
        match self.after_deaths {
            AfterDeaths::SpeakOrder => self.enter_speak_order(),
            AfterDeaths::Night => self.enter_night(),
        }
    }

    fn advance_death(&mut self) {
        while self.winner.is_none() {
            let Some(task) = self.death_tasks.front().cloned() else { return };
            match self.death_stage {
                DeathStage::LastWords => {
                    self.death_stage = DeathStage::Hunter;
                    if task.last_words {
                        let req = DecisionRequest::Speak { kind: SpeechKind::LastWords };
                        self.pending.insert(task.seat, req);
                        return;
                    }
                }
                DeathStage::Hunter => {
                    self.death_stage = DeathStage::Badge;
                    if self.role_of(task.seat) == Role::Hunter && task.cause != DeathCause::Poison {
                        let candidates = self.alive_seats();
                        self.pending.insert(task.seat, DecisionRequest::HunterShoot { candidates });
                        return;
                    }
                }
                DeathStage::Badge => {
                    self.death_stage = DeathStage::Done;
                    if self.sheriff == Some(task.seat) {
                        let candidates = self.alive_seats();
                        self.pending.insert(task.seat, DecisionRequest::BadgeTransfer { candidates });
                        return;
                    }
                }
                DeathStage::Done => {
                    self.death_tasks.pop_front();
                    self.death_stage = DeathStage::LastWords;
                }
            }
        }
    }

    fn enter_speak_order(&mut self) {
        self.begin(Step::SpeakOrder);
        if let Some(s) = self.living_sheriff() {
            self.pending.insert(s, DecisionRequest::ChooseSpeakOrder);
        }
    }

    fn living_sheriff(&self) -> Option<SeatId> {
        self.sheriff.filter(|s| self.is_alive(*s))
    }

    fn resolve_speak_order(&mut self) {
        let order: Vec<(SeatId, SpeechKind)>;
        let dir;
        if let Some(sheriff) = self.living_sheriff() {
            // The sheriff picks a side and speaks last (归票).
            dir = match self.submitted.get(&sheriff) {
                Some(Decision::SpeakOrder(d)) => *d,
                _ => Direction::Clockwise,
            };
            order = self
                .ring_after(sheriff, dir)
                .into_iter()
                .map(|s| (s, if s == sheriff { SpeechKind::SheriffSummary } else { SpeechKind::Normal }))
                .collect();
        } else {
            dir = if self.rng.gen_bool(0.5) { Direction::Clockwise } else { Direction::CounterClockwise };
            let mut seats = match self.last_night_deaths.first() {
                // Start next to the (first) victim.
                Some(dead) => self.ring_after(*dead, dir),
                None => {
                    let alive = self.alive_seats();
                    let first = alive[self.rng.gen_range(0..alive.len())];
                    let mut ring = self.ring_after(first, dir);
                    ring.rotate_right(1);
                    ring
                }
            };
            seats.dedup();
            order = seats.into_iter().map(|s| (s, SpeechKind::Normal)).collect();
        }
        let first = order[0].0;
        self.emit(Visibility::Public, EventKind::SpeakOrderChosen { first, dir });
        self.speak_queue = order.into();
        self.begin(Step::Speech);
        self.advance_speech();
    }

    fn resolve_exile_vote(&mut self) {
        let first_round = self.step == Step::ExileVote;
        let kind = if first_round { VoteKind::Exile } else { VoteKind::ExilePk };
        let (winner, tied) = self.tally(kind);
        if first_round && tied.len() > 1 {
            self.ballot = tied.clone();
            self.speak_queue = tied.iter().map(|s| (*s, SpeechKind::ExilePk)).collect();
            self.begin(Step::ExilePkSpeech);
            self.advance_speech();
        } else {
            self.exile(winner);
        }
    }

    fn exile(&mut self, target: Option<SeatId>) {
        self.emit(Visibility::Public, EventKind::Exiled { seat: target });
        let Some(seat) = target else { return self.enter_night() };
        self.alive[seat.index()] = false;
        if self.check_win() {
            return;
        }
        self.death_tasks = [DeathTask { seat, cause: DeathCause::Exile, last_words: true }].into();
        self.after_deaths = AfterDeaths::Night;
        self.continue_deaths();
    }

    /// If both sides are wiped out at once (e.g. poison + knife), the good side wins.
    fn check_win(&mut self) -> bool {
        let count = |cat: Category| {
            (0..self.players()).filter(|&i| self.alive[i] && self.roles[i].category() == cat).count()
        };
        let has = |cat: Category| self.roles.iter().any(|r| r.category() == cat);
        let (wolves, gods, villagers) = (count(Category::Wolf), count(Category::God), count(Category::Villager));
        let wolves_win = match self.cfg.rules.win_rule {
            WinRule::KillSide => (has(Category::God) && gods == 0) || (has(Category::Villager) && villagers == 0),
            WinRule::KillAll => gods + villagers == 0,
        };
        let winner = if wolves == 0 {
            Faction::Good
        } else if wolves_win {
            Faction::Wolf
        } else {
            return false;
        };

        self.winner = Some(winner);
        self.step = Step::GameOver;
        self.pending.clear();
        self.speak_queue.clear();
        self.death_tasks.clear();
        let roles = (0..self.players()).map(|i| (SeatId::from_index(i), self.roles[i])).collect();
        self.emit(Visibility::Public, EventKind::GameOver { winner, roles });
        true
    }
}

fn invalid(seat: SeatId, reason: &str) -> RuleError {
    RuleError::InvalidDecision { seat, reason: reason.to_string() }
}

fn pick_ok(target: &Option<SeatId>, candidates: &[SeatId]) -> bool {
    target.is_none_or(|t| candidates.contains(&t))
}

fn validate(seat: SeatId, req: &DecisionRequest, d: &Decision) -> Result<(), RuleError> {
    use Decision as D;
    use DecisionRequest as R;
    let ok = match (req, d) {
        (R::WolfKill { candidates, .. }, D::WolfKill(t)) => pick_ok(t, candidates),
        (R::WitchAct { can_save, can_poison, poison_candidates, .. }, D::Witch(action)) => match action {
            WitchAction::Save => *can_save,
            WitchAction::Poison(t) => *can_poison && poison_candidates.contains(t),
            WitchAction::Pass => true,
        },
        (R::SeerCheck { candidates }, D::SeerCheck(t)) => pick_ok(t, candidates),
        (R::SheriffSignup, D::SheriffSignup(_)) => true,
        (R::SheriffWithdrawWindow, D::SheriffWithdraw | D::Stay) => true,
        (R::Speak { .. }, D::Speak(_)) => true,
        (R::Vote { candidates, .. }, D::Vote(t)) => pick_ok(t, candidates),
        (R::HunterShoot { candidates }, D::HunterShoot(t)) => pick_ok(t, candidates),
        (R::BadgeTransfer { candidates }, D::BadgeTransfer(t)) => pick_ok(t, candidates),
        (R::ChooseSpeakOrder, D::SpeakOrder(_)) => true,
        _ => false,
    };
    if ok {
        Ok(())
    } else {
        Err(invalid(seat, &format!("{d:?} does not answer {req:?}")))
    }
}
