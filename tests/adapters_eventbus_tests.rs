#[cfg(test)]
mod tests {
    use terminal_pixel_dungeon::hero_adapter::{HeroAdapter, StatsAdapter};
    use terminal_pixel_dungeon::event_bus::EventBus;
    use hero::Hero;

    #[test]
    fn hero_stats_adapter_roundtrip() {
        let hero = Hero::with_seed(hero::class::Class::Warrior, 42);
        let stats = hero.to_stats();
        let hero2 = stats.to_hero();
        assert_eq!(hero2.level, hero.level);
    }

    #[test]
    fn event_bus_publish_subscribe() {
        use terminal_pixel_dungeon::event_bus::GameEvent;
        let bus = EventBus::new();
        let result = bus.publish(&GameEvent::PlayerMoved { x: 1, y: 2 });
        assert!(result.is_ok());
    }
}
