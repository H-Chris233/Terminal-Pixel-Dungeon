#[cfg(test)]
mod tests {
    use hero::Hero;
    use terminal_pixel_dungeon::event_bus::EventBus;
    use terminal_pixel_dungeon::hero_adapter::{HeroAdapter, StatsAdapter};

    #[test]
    fn hero_stats_adapter_roundtrip() {
        let hero = Hero::with_seed(hero::class::Class::Warrior, 42);
        let stats = hero.to_stats();
        let hero2 = stats.to_hero();
        assert_eq!(hero2.level, hero.level);
        assert_eq!(hero2.class, hero.class);
    }

    #[test]
    fn event_bus_publish_subscribe() {
        use terminal_pixel_dungeon::event_bus::GameEvent;
        let mut bus = EventBus::new();
        bus.publish(GameEvent::PlayerTurnStarted);
        assert_eq!(bus.len(), 1);
    }
}
