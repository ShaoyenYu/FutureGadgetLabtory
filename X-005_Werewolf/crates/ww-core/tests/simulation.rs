use ww_core::sim::simulate;
use ww_core::{Category, EventKind, Faction, Game, GameConfig, Role, SeatId};

const SEEDS_PER_SIZE: u64 = 1000;

fn check_invariants(g: &Game) {
    let events = g.events();
    for (i, e) in events.iter().enumerate() {
        assert_eq!(e.seq, i as u64, "event seq must be contiguous");
    }
    let game_overs = events.iter().filter(|e| matches!(e.kind, EventKind::GameOver { .. })).count();
    assert_eq!(game_overs, 1);
    let EventKind::GameOver { winner, .. } = &events.last().unwrap().kind else {
        panic!("GameOver must be the last event");
    };
    assert_eq!(Some(*winner), g.winner());
    assert!(g.pending().is_empty());

    let alive = |cat: Category| g.alive_seats().iter().filter(|s| g.role_of(**s).category() == cat).count();
    match winner {
        Faction::Good => assert_eq!(alive(Category::Wolf), 0),
        Faction::Wolf => {
            assert!(alive(Category::Wolf) > 0);
            assert!(alive(Category::God) == 0 || alive(Category::Villager) == 0);
        }
    }
}

/// Semantic isolation check, independent of how events are tagged: a seat must
/// never observe information its role is not entitled to.
fn check_isolation(g: &Game) {
    for i in 0..g.players() {
        let me = SeatId::from_index(i);
        let role = g.role_of(me);
        for obs in g.observations_for(me) {
            match obs.kind() {
                EventKind::RoleAssigned { seat, .. } => assert_eq!(*seat, me, "saw another seat's role"),
                EventKind::WolfTeammates { .. }
                | EventKind::WolfChat { .. }
                | EventKind::WolfVoteCast { .. }
                | EventKind::WolfTarget { .. } => assert_eq!(role, Role::Werewolf, "wolf info leaked to {me}"),
                EventKind::WitchInfo { .. } | EventKind::WitchSaved { .. } | EventKind::WitchPoisoned { .. } => {
                    assert_eq!(role, Role::Witch, "witch info leaked to {me}")
                }
                EventKind::SeerResult { .. } => assert_eq!(role, Role::Seer, "seer result leaked to {me}"),
                EventKind::NightResolved { .. } => panic!("god-only event leaked to {me}"),
                _ => {}
            }
        }
    }
}

#[test]
fn random_games_finish_for_every_board_size() {
    for players in GameConfig::MIN_PLAYERS..=GameConfig::MAX_PLAYERS {
        let cfg = GameConfig::preset(players).unwrap();
        let mut wins = [0u32; 2];
        for seed in 0..SEEDS_PER_SIZE {
            let g = simulate(cfg.clone(), seed).unwrap_or_else(|e| panic!("{players}p seed {seed}: {e}"));
            check_invariants(&g);
            check_isolation(&g);
            wins[(g.winner() == Some(Faction::Wolf)) as usize] += 1;
        }
        // Both sides must be able to win under random play.
        assert!(wins[0] > 0 && wins[1] > 0, "{players}p: good {} / wolf {}", wins[0], wins[1]);
    }
}

#[test]
fn same_seed_replays_identically() {
    let cfg = GameConfig::preset(9).unwrap();
    for seed in [1, 42, 999] {
        let a = serde_json::to_string(simulate(cfg.clone(), seed).unwrap().events()).unwrap();
        let b = serde_json::to_string(simulate(cfg.clone(), seed).unwrap().events()).unwrap();
        assert_eq!(a, b);
    }
}

#[test]
fn presets_cover_six_to_twelve() {
    assert!(GameConfig::preset(5).is_err());
    assert!(GameConfig::preset(13).is_err());
    for n in 6..=12 {
        assert_eq!(GameConfig::preset(n).unwrap().players(), n);
    }
    let nine = GameConfig::preset(9).unwrap();
    let wolves = nine.roles.iter().filter(|r| **r == Role::Werewolf).count();
    assert_eq!(wolves, 3);
}
