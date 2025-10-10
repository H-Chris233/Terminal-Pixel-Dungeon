#[cfg(test)]
mod tests {
    use terminal_pixel_dungeon::hero_adapter::{HeroAdapter, StatsAdapter, InventoryAdapter, BagAdapter};
    use terminal_pixel_dungeon::event_bus::{InMemoryBus, EventBus};
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
        let bus = InMemoryBus::<String>::new();
        bus.publish("hello".to_string());
        let mut sub = bus.subscribe();
        assert_eq!(sub.next(), Some("hello".to_string()));
    }
}
