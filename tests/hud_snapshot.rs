use ratatui::{Terminal, backend::TestBackend};
use terminal_pixel_dungeon::ecs::{
    Actor, Color, ECSWorld, Energy, Faction, Hunger, Player, PlayerProgress, Stats,
    StatusFeedEntry, TurnHudState, TurnQueueEntry, Wealth,
};
use terminal_pixel_dungeon::render::hud::HudRenderer;

#[test]
fn hud_turn_overlay_snapshot() {
    let mut ecs_world = ECSWorld::new();

    let player_entity = ecs_world.world.spawn((
        Actor {
            name: "Player".to_string(),
            faction: Faction::Player,
        },
        Player,
        Stats {
            hp: 72,
            max_hp: 100,
            attack: 12,
            defense: 6,
            accuracy: 80,
            evasion: 20,
            level: 3,
            experience: 180,
        },
        Wealth::new(128),
        Hunger::new(4),
        PlayerProgress::new(10, "Warrior".to_string()),
    ));

    let enemy_entity = ecs_world.world.spawn((
        Actor {
            name: "Goblin".to_string(),
            faction: Faction::Enemy,
        },
        Energy {
            current: 60,
            max: 100,
            regeneration_rate: 5,
        },
    ));

    let mut hud_state = TurnHudState::default();
    hud_state.turn_count = 42;

    let player_entry = TurnQueueEntry {
        entity: player_entity.id() as u32,
        name: "Player".to_string(),
        faction: Faction::Player,
        energy: 100,
        max_energy: 100,
        regen: 10,
        eta: 0,
        queued_action: None,
    };

    let enemy_entry = TurnQueueEntry {
        entity: enemy_entity.id() as u32,
        name: "Goblin".to_string(),
        faction: Faction::Enemy,
        energy: 60,
        max_energy: 100,
        regen: 5,
        eta: 8,
        queued_action: Some("排队".to_string()),
    };

    hud_state.current_actor = Some(player_entry.clone());
    hud_state.queue = vec![player_entry, enemy_entry];
    hud_state.status_feed = vec![
        StatusFeedEntry {
            message: "造成 5 点伤害".to_string(),
            color: Color::Red,
        },
        StatusFeedEntry {
            message: "饱食度下降到 3".to_string(),
            color: Color::Yellow,
        },
    ];

    ecs_world.resources.game_state.turn_overlay = hud_state;

    let hud = HudRenderer::new();
    let backend = TestBackend::new(60, 12);
    let mut terminal = Terminal::new(backend).expect("failed to create terminal");

    terminal
        .draw(|frame| {
            let area = frame.area();
            hud.render(
                frame,
                area,
                &ecs_world.world,
                &ecs_world.resources.game_state,
            );
        })
        .expect("render failure");

    let buffer = terminal.backend_mut().buffer().clone();

    let mut lines = Vec::new();
    let height = buffer.area().height as usize;
    let width = buffer.area().width as usize;
    for y in 0..height {
        let mut line = String::new();
        for x in 0..width {
            let cell = &buffer[(x as u16, y as u16)];
            line.push_str(cell.symbol());
        }
        lines.push(line.trim_end().to_string());
    }

    assert!(
        lines
            .iter()
            .any(|line| line.contains("回 合") && line.contains("42")),
        "turn counter not rendered"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("当 前") && line.contains("Player")),
        "active actor summary missing"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("████████████ 100/100")),
        "energy bar for active actor missing"
    );
    assert!(
        lines.iter().any(|line| line.contains("▶ Player")),
        "queue marker for player missing"
    );
    assert!(
        lines.iter().any(|line| line.contains("eta 8")),
        "enemy eta annotation missing"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("饱 食 度 下 降 到  3")),
        "status feed not showing hunger event"
    );
}
