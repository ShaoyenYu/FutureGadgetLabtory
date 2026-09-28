//! Scripted scenarios on a fixed 9-player layout:
//! 1-3 wolves, 4 seer, 5 witch, 6 hunter, 7-9 villagers.

use ww_core::{
    Decision, DecisionRequest, Direction, EventKind, Faction, Game, GameConfig, NightStep, Role, RuleError,
    RuleOptions, SeatId, SpeechKind, Step, WitchAction,
};

use Role::{Hunter, Seer, Villager as V, Werewolf as W, Witch};

const LAYOUT: [Role; 9] = [W, W, W, Seer, Witch, Hunter, V, V, V];

fn s(n: u8) -> SeatId {
    SeatId(n)
}

fn game(sheriff: bool) -> Game {
    let mut cfg = GameConfig::preset(9).unwrap();
    cfg.rules.sheriff = sheriff;
    Game::with_roles(cfg, LAYOUT.to_vec(), 7).unwrap()
}

fn act(g: &mut Game, seat: u8, d: Decision) {
    g.submit(s(seat), d).unwrap_or_else(|e| panic!("{seat}: {e}"));
}

/// Plays one night: every living wolf knifes `kill`; witch/seer act if they can.
fn night(g: &mut Game, kill: u8, witch: WitchAction, check: Option<u8>) {
    assert_eq!(g.step(), Step::Night(NightStep::WolfKill));
    let wolves: Vec<SeatId> = g.pending().keys().copied().collect();
    for w in wolves {
        g.submit(w, Decision::WolfKill(Some(s(kill)))).unwrap();
    }
    if g.pending().contains_key(&s(5)) {
        act(g, 5, Decision::Witch(witch));
    }
    if g.pending().contains_key(&s(4)) {
        act(g, 4, Decision::SeerCheck(check.map(s)));
    }
}

/// Lets the clock run out until `pred` holds.
fn run_until(g: &mut Game, pred: impl Fn(Step) -> bool) {
    while !pred(g.step()) {
        assert!(!g.is_over(), "game ended while waiting");
        g.timeout();
    }
}

fn votes(g: &mut Game, ballots: &[(u8, Option<u8>)]) {
    let vote_step = g.step();
    let before = g.events().len();
    for (voter, target) in ballots {
        act(g, *voter, Decision::Vote(target.map(s)));
    }
    let closed = g.events()[before..].iter().any(|e| matches!(e.kind, EventKind::VoteResult { .. }));
    if !closed {
        assert_eq!(g.step(), vote_step);
        g.timeout(); // everyone else abstains
    }
}

fn has(g: &Game, pred: impl Fn(&EventKind) -> bool) -> bool {
    g.events().iter().any(|e| pred(&e.kind))
}

#[test]
fn witch_save_gives_a_peaceful_night() {
    let mut g = game(false);
    for w in 1..=3 {
        act(&mut g, w, Decision::WolfKill(Some(s(7))));
    }
    assert_eq!(
        g.pending().get(&s(5)),
        Some(&DecisionRequest::WitchAct {
            victim: Some(s(7)),
            can_save: true,
            can_poison: true,
            poison_candidates: vec![s(1), s(2), s(3), s(4), s(6), s(7), s(8), s(9)],
        })
    );
    act(&mut g, 5, Decision::Witch(WitchAction::Save));
    act(&mut g, 4, Decision::SeerCheck(Some(s(1))));

    assert!(has(&g, |k| matches!(k, EventKind::DeathsAnnounced { seats } if seats.is_empty())));
    let seer_saw_wolf = g
        .observations_for(s(4))
        .iter()
        .any(|o| matches!(o.kind(), EventKind::SeerResult { target, is_wolf: true } if *target == s(1)));
    assert!(seer_saw_wolf);
    assert_eq!(g.step(), Step::Speech);
    assert_eq!(g.alive_seats().len(), 9);
}

#[test]
fn wolf_majority_decides_and_ties_go_to_latest_vote() {
    let mut g = game(false);
    act(&mut g, 1, Decision::WolfKill(Some(s(7))));
    act(&mut g, 2, Decision::WolfKill(Some(s(8))));
    act(&mut g, 3, Decision::WolfKill(Some(s(8))));
    let DecisionRequest::WitchAct { victim, .. } = &g.pending()[&s(5)] else { panic!() };
    assert_eq!(*victim, Some(s(8)));

    let mut g = game(false);
    act(&mut g, 1, Decision::WolfKill(Some(s(7))));
    act(&mut g, 2, Decision::WolfKill(Some(s(8))));
    act(&mut g, 3, Decision::WolfKill(None));
    let DecisionRequest::WitchAct { victim, .. } = &g.pending()[&s(5)] else { panic!() };
    assert_eq!(*victim, None, "three-way tie: the latest vote (空刀) wins");
}

#[test]
fn witch_cannot_self_save_after_the_first_night() {
    let mut g = game(false);
    night(&mut g, 7, WitchAction::Pass, Some(1));
    run_until(&mut g, |st| st == Step::ExileVote);
    g.timeout(); // nobody votes → no exile → night 2
    assert_eq!(g.round(), 2);

    for w in 1..=3 {
        act(&mut g, w, Decision::WolfKill(Some(s(5))));
    }
    let DecisionRequest::WitchAct { victim, can_save, .. } = &g.pending()[&s(5)] else { panic!() };
    assert_eq!((*victim, *can_save), (Some(s(5)), false));
    let before = g.events().len();
    let err = g.submit(s(5), Decision::Witch(WitchAction::Save)).unwrap_err();
    assert!(matches!(err, RuleError::InvalidDecision { .. }));
    assert_eq!(g.events().len(), before, "rejected decisions leave no trace");
}

#[test]
fn witch_may_self_save_on_the_first_night() {
    let mut g = game(false);
    for w in 1..=3 {
        act(&mut g, w, Decision::WolfKill(Some(s(5))));
    }
    let DecisionRequest::WitchAct { can_save, .. } = &g.pending()[&s(5)] else { panic!() };
    assert!(*can_save);
}

#[test]
fn poisoned_hunter_cannot_shoot() {
    let mut g = game(false);
    night(&mut g, 7, WitchAction::Poison(s(6)), None);
    assert!(has(&g, |k| matches!(k, EventKind::DeathsAnnounced { seats } if seats == &vec![s(6), s(7)])));
    assert_eq!(g.pending().get(&s(6)), Some(&DecisionRequest::Speak { kind: SpeechKind::LastWords }));
    act(&mut g, 6, Decision::Speak("我是猎人，被毒了".into()));
    assert_eq!(g.pending().get(&s(7)), Some(&DecisionRequest::Speak { kind: SpeechKind::LastWords }));
    assert!(!g.pending().contains_key(&s(6)));
}

#[test]
fn knifed_hunter_shoots_and_the_target_gets_last_words() {
    let mut g = game(false);
    night(&mut g, 6, WitchAction::Pass, None);
    act(&mut g, 6, Decision::Speak("猎人，带走1号".into()));
    assert!(matches!(g.pending().get(&s(6)), Some(DecisionRequest::HunterShoot { .. })));
    act(&mut g, 6, Decision::HunterShoot(Some(s(1))));
    assert!(has(&g, |k| matches!(k, EventKind::HunterShot { hunter, target } if *hunter == s(6) && *target == s(1))));
    assert!(!g.is_alive(s(1)));
    assert_eq!(g.pending().get(&s(1)), Some(&DecisionRequest::Speak { kind: SpeechKind::LastWords }));
}

#[test]
fn no_last_words_for_night_deaths_after_night_one() {
    let mut g = game(false);
    night(&mut g, 7, WitchAction::Save, None);
    run_until(&mut g, |st| st == Step::ExileVote);
    g.timeout();
    night(&mut g, 8, WitchAction::Pass, None);
    assert!(!g.is_alive(s(8)));
    assert_eq!(g.step(), Step::Speech, "went straight to speeches");
}

#[test]
fn exile_tie_twice_means_nobody_leaves() {
    let mut g = game(false);
    night(&mut g, 7, WitchAction::Save, None);
    run_until(&mut g, |st| st == Step::ExileVote);
    votes(&mut g, &[(1, Some(7)), (2, Some(7)), (3, Some(8)), (4, Some(8))]);
    assert_eq!(g.step(), Step::ExilePkSpeech);
    run_until(&mut g, |st| st == Step::ExilePkVote);
    assert!(!g.pending().contains_key(&s(7)) && !g.pending().contains_key(&s(8)), "PK seats don't vote");
    votes(&mut g, &[(1, Some(7)), (2, Some(7)), (3, Some(8)), (4, Some(8))]);
    assert!(has(&g, |k| matches!(k, EventKind::Exiled { seat: None })));
    assert_eq!(g.step(), Step::Night(NightStep::WolfKill));
    assert_eq!(g.alive_seats().len(), 9);
}

#[test]
fn sheriff_weight_breaks_an_exile_tie() {
    let mut g = game(true);
    night(&mut g, 7, WitchAction::Save, None);
    assert_eq!(g.step(), Step::SheriffSignup);
    for seat in 1..=9 {
        act(&mut g, seat, Decision::SheriffSignup(seat == 4 || seat == 8));
    }
    run_until(&mut g, |st| st == Step::SheriffWithdraw);
    act(&mut g, 4, Decision::Stay);
    act(&mut g, 8, Decision::Stay);
    assert_eq!(g.step(), Step::SheriffVote);
    assert!(!g.pending().contains_key(&s(4)), "candidates cannot vote");
    votes(&mut g, &[(1, Some(8)), (2, Some(8)), (3, Some(8)), (5, Some(4)), (6, Some(4)), (7, Some(4)), (9, Some(4))]);
    assert_eq!(g.sheriff(), Some(s(4)));

    assert_eq!(g.pending().get(&s(4)), Some(&DecisionRequest::ChooseSpeakOrder));
    act(&mut g, 4, Decision::SpeakOrder(Direction::Clockwise));
    assert!(has(&g, |k| matches!(k, EventKind::SpeakOrderChosen { first, .. } if *first == s(5))));
    // Sheriff speaks last and sums up.
    while !g.pending().contains_key(&s(4)) {
        g.timeout();
    }
    assert_eq!(g.pending().get(&s(4)), Some(&DecisionRequest::Speak { kind: SpeechKind::SheriffSummary }));
    g.timeout();

    // 3 votes vs 3 votes, but the sheriff's counts 1.5.
    votes(&mut g, &[(4, Some(1)), (5, Some(1)), (6, Some(1)), (1, Some(7)), (2, Some(7)), (3, Some(7))]);
    assert!(has(&g, |k| matches!(k, EventKind::Exiled { seat: Some(x) } if *x == s(1))));
}

#[test]
fn everyone_running_for_sheriff_voids_the_election() {
    let mut g = game(true);
    night(&mut g, 7, WitchAction::Save, None);
    for seat in 1..=9 {
        act(&mut g, seat, Decision::SheriffSignup(true));
    }
    assert!(has(&g, |k| matches!(k, EventKind::SheriffElected { seat: None })));
    assert_eq!(g.sheriff(), None);
}

#[test]
fn mid_speech_withdrawal_leaves_a_single_winner() {
    let mut g = game(true);
    night(&mut g, 7, WitchAction::Save, None);
    for seat in 1..=9 {
        act(&mut g, seat, Decision::SheriffSignup(seat == 4 || seat == 8));
    }
    assert_eq!(g.step(), Step::SheriffSpeech);
    act(&mut g, 8, Decision::SheriffWithdraw);
    run_until(&mut g, |st| st != Step::SheriffSpeech);
    assert!(has(&g, |k| matches!(k, EventKind::SheriffWithdrew { seat } if *seat == s(8))));
    assert_eq!(g.sheriff(), Some(s(4)));
}

#[test]
fn sheriff_killed_on_night_one_passes_the_badge() {
    let mut g = game(true);
    night(&mut g, 8, WitchAction::Pass, None);
    // The night's victim is not yet announced and can still run.
    for seat in 1..=9 {
        act(&mut g, seat, Decision::SheriffSignup(seat == 8));
    }
    assert_eq!(g.sheriff(), Some(s(8)));
    assert!(!g.is_alive(s(8)));
    act(&mut g, 8, Decision::Speak("警徽给4号".into()));
    assert!(matches!(g.pending().get(&s(8)), Some(DecisionRequest::BadgeTransfer { .. })));
    act(&mut g, 8, Decision::BadgeTransfer(Some(s(4))));
    assert_eq!(g.sheriff(), Some(s(4)));
    assert_eq!(g.pending().get(&s(4)), Some(&DecisionRequest::ChooseSpeakOrder));
}

#[test]
fn exiling_the_last_wolf_ends_the_game() {
    use Role::{Seer, Villager as V, Werewolf as W, Witch};
    let roles = vec![W, Seer, Witch, V, V, V];
    let cfg = GameConfig::custom(roles.clone(), RuleOptions { sheriff: false, ..Default::default() }).unwrap();
    let mut g = Game::with_roles(cfg, roles, 1).unwrap();
    act(&mut g, 1, Decision::WolfKill(Some(s(4))));
    act(&mut g, 3, Decision::Witch(WitchAction::Save));
    act(&mut g, 2, Decision::SeerCheck(Some(s(1))));
    run_until(&mut g, |st| st == Step::ExileVote);
    votes(&mut g, &[(2, Some(1)), (3, Some(1)), (4, Some(1))]);
    assert_eq!(g.winner(), Some(Faction::Good));
    assert!(matches!(g.events().last().unwrap().kind, EventKind::GameOver { winner: Faction::Good, .. }));
    assert!(g.pending().is_empty());
}

#[test]
fn wolves_win_once_every_god_is_dead() {
    use Role::{Seer, Villager as V, Werewolf as W, Witch};
    let roles = vec![W, W, Seer, Witch, V, V];
    let cfg = GameConfig::custom(roles.clone(), RuleOptions { sheriff: false, ..Default::default() }).unwrap();
    let mut g = Game::with_roles(cfg, roles, 1).unwrap();
    act(&mut g, 1, Decision::WolfKill(Some(s(4))));
    act(&mut g, 2, Decision::WolfKill(Some(s(4))));
    act(&mut g, 4, Decision::Witch(WitchAction::Pass));
    act(&mut g, 3, Decision::SeerCheck(Some(s(1))));
    run_until(&mut g, |st| st == Step::ExileVote);
    votes(&mut g, &[(1, Some(3)), (2, Some(3)), (5, Some(3))]);
    assert_eq!(g.winner(), Some(Faction::Wolf));
}

#[test]
fn invalid_decisions_are_rejected_without_side_effects() {
    let mut g = game(false);
    let before = g.events().len();
    assert_eq!(g.submit(s(7), Decision::WolfKill(Some(s(8)))).unwrap_err(), RuleError::NotPending(s(7)));
    assert!(matches!(g.submit(s(1), Decision::Vote(Some(s(8)))), Err(RuleError::InvalidDecision { .. })));
    assert!(matches!(g.submit(s(7), Decision::WolfChat("hi".into())), Err(RuleError::InvalidDecision { .. })));
    assert_eq!(g.submit(s(0), Decision::Stay).unwrap_err(), RuleError::UnknownSeat(s(0)));
    assert_eq!(g.submit(s(10), Decision::Stay).unwrap_err(), RuleError::UnknownSeat(s(10)));
    assert_eq!(g.events().len(), before);
}

#[test]
fn wolf_chat_reaches_only_wolves() {
    let mut g = game(false);
    act(&mut g, 1, Decision::WolfChat("刀7号".into()));
    let saw_chat = |seat: u8| {
        g.observations_for(s(seat)).iter().any(|o| matches!(o.kind(), EventKind::WolfChat { .. }))
    };
    assert!(saw_chat(2) && saw_chat(3));
    assert!(!saw_chat(4) && !saw_chat(7));
}

#[test]
fn dead_witch_step_is_still_announced() {
    let mut g = game(false);
    night(&mut g, 5, WitchAction::Pass, None);
    run_until(&mut g, |st| st == Step::ExileVote);
    g.timeout();
    let start = g.events().len();
    night(&mut g, 7, WitchAction::Pass, None);
    let announced_witch = g.events()[start..]
        .iter()
        .any(|e| matches!(e.kind, EventKind::StepStarted { step: Step::Night(NightStep::Witch) }));
    assert!(announced_witch, "the witch step is called out even though she is dead");
}
