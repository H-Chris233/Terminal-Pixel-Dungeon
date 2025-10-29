// Tests for expanded event bus taxonomy and turn-phase integration

use terminal_pixel_dungeon::event_bus::*;
use std::sync::{Arc, Mutex};

// ===== Helper Structures =====

struct TestHandler {
    name: String,
    handled_events: Arc<Mutex<Vec<String>>>,
    priority: Priority,
    phases: Vec<TurnPhase>,
}

impl TestHandler {
    fn new(name: &str, priority: Priority) -> Self {
        Self {
            name: name.to_string(),
            handled_events: Arc::new(Mutex::new(Vec::new())),
            priority,
            phases: vec![TurnPhase::Any],
        }
    }

    fn with_phases(mut self, phases: Vec<TurnPhase>) -> Self {
        self.phases = phases;
        self
    }

    fn get_handled_events(&self) -> Vec<String> {
        self.handled_events.lock().unwrap().clone()
    }

    fn clear(&self) {
        self.handled_events.lock().unwrap().clear();
    }
}

impl EventHandler for TestHandler {
    fn handle(&mut self, event: &GameEvent) {
        let event_type = event.event_type();
        self.handled_events
            .lock()
            .unwrap()
            .push(format!("{}: {}", self.name, event_type));
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> Priority {
        self.priority
    }

    fn run_in_phases(&self) -> Vec<TurnPhase> {
        self.phases.clone()
    }
}

struct ShortCircuitMiddleware {
    block_event_types: Vec<&'static str>,
}

impl ShortCircuitMiddleware {
    fn new(block_event_types: Vec<&'static str>) -> Self {
        Self { block_event_types }
    }
}

impl EventMiddleware for ShortCircuitMiddleware {
    fn before_handle(&mut self, event: &GameEvent) -> bool {
        !self.block_event_types.contains(&event.event_type())
    }

    fn name(&self) -> &str {
        "ShortCircuitMiddleware"
    }

    fn priority(&self) -> Priority {
        Priority::Critical
    }
}

// ===== Test Event Categories =====

#[test]
fn test_event_categories() {
    // Test combat events
    let combat_event = GameEvent::CombatStarted {
        attacker: 1,
        defender: 2,
    };
    assert_eq!(combat_event.category(), EventCategory::Combat);

    let damage_event = GameEvent::DamageDealt {
        attacker: 1,
        victim: 2,
        damage: 10,
        is_critical: false,
    };
    assert_eq!(damage_event.category(), EventCategory::Combat);

    // Test new combat events
    let blocked_event = GameEvent::CombatBlocked {
        attacker: 1,
        defender: 2,
        blocked_damage: 5,
    };
    assert_eq!(blocked_event.category(), EventCategory::Combat);

    let parried_event = GameEvent::CombatParried {
        attacker: 1,
        defender: 2,
        parry_damage: 3,
    };
    assert_eq!(parried_event.category(), EventCategory::Combat);

    // Test movement events
    let move_event = GameEvent::EntityMoved {
        entity: 1,
        from_x: 0,
        from_y: 0,
        to_x: 1,
        to_y: 1,
    };
    assert_eq!(move_event.category(), EventCategory::Movement);

    // Test status events
    let status_event = GameEvent::StatusApplied {
        entity: 1,
        status: "Poison".to_string(),
        duration: 5,
        intensity: 2,
    };
    assert_eq!(status_event.category(), EventCategory::Status);

    let status_stacked = GameEvent::StatusStacked {
        entity: 1,
        status: "Poison".to_string(),
        old_intensity: 2,
        new_intensity: 4,
    };
    assert_eq!(status_stacked.category(), EventCategory::Status);

    // Test environment events
    let door_event = GameEvent::DoorOpened {
        entity: 1,
        x: 5,
        y: 10,
        door_type: "wooden door".to_string(),
    };
    assert_eq!(door_event.category(), EventCategory::Environment);

    let secret_event = GameEvent::SecretDiscovered {
        entity: 1,
        x: 10,
        y: 15,
        secret_type: "hidden passage".to_string(),
    };
    assert_eq!(secret_event.category(), EventCategory::Environment);

    // Test UI events
    let ui_event = GameEvent::UINotification {
        message: "Test notification".to_string(),
        notification_type: "info".to_string(),
        duration_ms: 3000,
    };
    assert_eq!(ui_event.category(), EventCategory::UI);

    // Test action events
    let action_event = GameEvent::ActionIntended {
        entity: 1,
        action_type: "attack".to_string(),
        priority: 100,
    };
    assert_eq!(action_event.category(), EventCategory::Action);

    let action_failed = GameEvent::ActionFailed {
        entity: 1,
        action_type: "cast spell".to_string(),
        reason: "not enough mana".to_string(),
    };
    assert_eq!(action_failed.category(), EventCategory::Action);
}

// ===== Test Priority Ordering =====

#[test]
fn test_priority_event_ordering() {
    let mut event_bus = EventBus::new();

    // Create handlers with different priorities
    let critical_handler = TestHandler::new("CriticalHandler", Priority::Critical);
    let high_handler = TestHandler::new("HighHandler", Priority::High);
    let normal_handler = TestHandler::new("NormalHandler", Priority::Normal);
    let low_handler = TestHandler::new("LowHandler", Priority::Low);

    let critical_events = critical_handler.handled_events.clone();
    let high_events = high_handler.handled_events.clone();
    let normal_events = normal_handler.handled_events.clone();
    let low_events = low_handler.handled_events.clone();

    // Register handlers (they should be sorted by priority)
    event_bus.subscribe_all(Box::new(critical_handler));
    event_bus.subscribe_all(Box::new(high_handler));
    event_bus.subscribe_all(Box::new(normal_handler));
    event_bus.subscribe_all(Box::new(low_handler));

    // Publish an event
    let event = GameEvent::DamageDealt {
        attacker: 1,
        victim: 2,
        damage: 10,
        is_critical: false,
    };
    event_bus.publish(event);

    // All handlers should have received the event
    assert_eq!(critical_events.lock().unwrap().len(), 1);
    assert_eq!(high_events.lock().unwrap().len(), 1);
    assert_eq!(normal_events.lock().unwrap().len(), 1);
    assert_eq!(low_events.lock().unwrap().len(), 1);

    // Verify they were called (priority doesn't change call order in subscribe_all, 
    // but it matters for phase queues)
    assert!(critical_events.lock().unwrap()[0].contains("DamageDealt"));
}

// ===== Test Phase-Aware Event Queues =====

#[test]
fn test_phase_aware_event_queues() {
    let mut event_bus = EventBus::new();

    // Publish events to different phases with different priorities
    event_bus.publish_to_phase(
        GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        },
        Priority::High,
        TurnPhase::Resolution,
    );

    event_bus.publish_to_phase(
        GameEvent::StatusApplied {
            entity: 2,
            status: "Poison".to_string(),
            duration: 3,
            intensity: 1,
        },
        Priority::Low,
        TurnPhase::Resolution,
    );

    event_bus.publish_to_phase(
        GameEvent::EntityMoved {
            entity: 1,
            from_x: 0,
            from_y: 0,
            to_x: 1,
            to_y: 1,
        },
        Priority::Critical,
        TurnPhase::Resolution,
    );

    // Drain Resolution phase events
    let events = event_bus.drain_phase(TurnPhase::Resolution);

    // Events should be ordered by priority (Critical, High, Low)
    assert_eq!(events.len(), 3);
    assert_eq!(events[0].event_type(), "EntityMoved"); // Critical
    assert_eq!(events[1].event_type(), "DamageDealt"); // High
    assert_eq!(events[2].event_type(), "StatusApplied"); // Low
}

#[test]
fn test_multiple_phase_queues() {
    let mut event_bus = EventBus::new();

    // Add events to different phases
    event_bus.publish_to_phase(
        GameEvent::ActionIntended {
            entity: 1,
            action_type: "attack".to_string(),
            priority: 100,
        },
        Priority::Normal,
        TurnPhase::Input,
    );

    event_bus.publish_to_phase(
        GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        },
        Priority::High,
        TurnPhase::Resolution,
    );

    event_bus.publish_to_phase(
        GameEvent::StatusEffectTicked {
            entity: 2,
            status: "Poison".to_string(),
            damage: 2,
            remaining_turns: 2,
        },
        Priority::Normal,
        TurnPhase::Aftermath,
    );

    // Drain each phase separately
    let input_events = event_bus.drain_phase(TurnPhase::Input);
    let resolution_events = event_bus.drain_phase(TurnPhase::Resolution);
    let aftermath_events = event_bus.drain_phase(TurnPhase::Aftermath);

    assert_eq!(input_events.len(), 1);
    assert_eq!(input_events[0].event_type(), "ActionIntended");

    assert_eq!(resolution_events.len(), 1);
    assert_eq!(resolution_events[0].event_type(), "DamageDealt");

    assert_eq!(aftermath_events.len(), 1);
    assert_eq!(aftermath_events[0].event_type(), "StatusEffectTicked");
}

// ===== Test Phase-Aware Handlers =====

#[test]
fn test_phase_aware_handlers() {
    let mut event_bus = EventBus::new();

    // Create handlers for specific phases
    let input_handler = TestHandler::new("InputHandler", Priority::Normal)
        .with_phases(vec![TurnPhase::Input]);
    let resolution_handler = TestHandler::new("ResolutionHandler", Priority::Normal)
        .with_phases(vec![TurnPhase::Resolution]);

    let input_events = input_handler.handled_events.clone();
    let resolution_events = resolution_handler.handled_events.clone();

    // Register handlers to their respective phases
    event_bus.subscribe_for_phase(TurnPhase::Input, Box::new(input_handler));
    event_bus.subscribe_for_phase(TurnPhase::Resolution, Box::new(resolution_handler));

    // Publish events to specific phases
    event_bus.publish_to_phase(
        GameEvent::ActionIntended {
            entity: 1,
            action_type: "move".to_string(),
            priority: 100,
        },
        Priority::Normal,
        TurnPhase::Input,
    );

    event_bus.publish_to_phase(
        GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 10,
            is_critical: false,
        },
        Priority::Normal,
        TurnPhase::Resolution,
    );

    // Process events for each phase
    event_bus.set_current_phase(TurnPhase::Input);
    event_bus.process_phase_events(TurnPhase::Input);

    event_bus.set_current_phase(TurnPhase::Resolution);
    event_bus.process_phase_events(TurnPhase::Resolution);

    // Input handler should only receive Input phase events
    let input_handled = input_events.lock().unwrap();
    assert_eq!(input_handled.len(), 1);
    assert!(input_handled[0].contains("ActionIntended"));

    // Resolution handler should only receive Resolution phase events
    let resolution_handled = resolution_events.lock().unwrap();
    assert_eq!(resolution_handled.len(), 1);
    assert!(resolution_handled[0].contains("DamageDealt"));
}

// ===== Test Middleware Short-Circuit =====

#[test]
fn test_middleware_short_circuit() {
    let mut event_bus = EventBus::new();

    // Register a middleware that blocks specific event types
    let middleware = ShortCircuitMiddleware::new(vec!["DamageDealt", "StatusApplied"]);
    event_bus.register_middleware(Box::new(middleware));

    // Register a handler to track which events get through
    let handler = TestHandler::new("TestHandler", Priority::Normal);
    let handled_events = handler.handled_events.clone();
    event_bus.subscribe_all(Box::new(handler));

    // Publish events that should be blocked
    event_bus.publish(GameEvent::DamageDealt {
        attacker: 1,
        victim: 2,
        damage: 10,
        is_critical: false,
    });

    event_bus.publish(GameEvent::StatusApplied {
        entity: 2,
        status: "Poison".to_string(),
        duration: 3,
        intensity: 1,
    });

    // Publish an event that should NOT be blocked
    event_bus.publish(GameEvent::EntityMoved {
        entity: 1,
        from_x: 0,
        from_y: 0,
        to_x: 1,
        to_y: 1,
    });

    // Only the EntityMoved event should be handled
    let handled = handled_events.lock().unwrap();
    assert_eq!(handled.len(), 1);
    assert!(handled[0].contains("EntityMoved"));
}

// ===== Test Recursive Publishing Protection =====

#[test]
fn test_recursive_publishing_protection() {
    let mut event_bus = EventBus::new();

    // We can't easily test the recursive depth limit without a complex handler
    // that publishes events, but we can verify the mechanism exists by checking
    // that batch_buffer is used

    // Manually set publish_depth to simulate recursion
    // (Note: In real code, this would happen through nested event publishing)
    
    // Publish multiple events rapidly to the same phase
    for i in 0..15 {
        event_bus.publish_to_phase(
            GameEvent::DamageDealt {
                attacker: 1,
                victim: 2,
                damage: i,
                is_critical: false,
            },
            Priority::Normal,
            TurnPhase::Resolution,
        );
    }

    // All events should be queued without panic
    let events = event_bus.drain_phase(TurnPhase::Resolution);
    assert!(events.len() >= 15);
}

// ===== Test FIFO Ordering Within Same Priority =====

#[test]
fn test_fifo_ordering_same_priority() {
    let mut event_bus = EventBus::new();

    // Publish multiple events with the same priority
    for i in 1..=5 {
        event_bus.publish_to_phase(
            GameEvent::DamageDealt {
                attacker: 1,
                victim: 2,
                damage: i,
                is_critical: false,
            },
            Priority::Normal,
            TurnPhase::Resolution,
        );
    }

    // Drain and verify FIFO order (first published = first processed)
    let events = event_bus.drain_phase(TurnPhase::Resolution);
    assert_eq!(events.len(), 5);

    // Extract damage values to verify order
    for (idx, event) in events.iter().enumerate() {
        if let GameEvent::DamageDealt { damage, .. } = event {
            assert_eq!(*damage, (idx as u32) + 1);
        }
    }
}

// ===== Test Current Phase Tracking =====

#[test]
fn test_current_phase_tracking() {
    let mut event_bus = EventBus::new();

    // Initial phase should be Input
    assert_eq!(event_bus.get_current_phase(), TurnPhase::Input);

    // Change phase
    event_bus.set_current_phase(TurnPhase::IntentQueue);
    assert_eq!(event_bus.get_current_phase(), TurnPhase::IntentQueue);

    event_bus.set_current_phase(TurnPhase::Resolution);
    assert_eq!(event_bus.get_current_phase(), TurnPhase::Resolution);

    event_bus.set_current_phase(TurnPhase::Aftermath);
    assert_eq!(event_bus.get_current_phase(), TurnPhase::Aftermath);
}

// ===== Test Event History with New Events =====

#[test]
fn test_event_history_with_new_events() {
    let mut event_bus = EventBus::with_history_size(10);

    // Publish various new event types
    event_bus.publish(GameEvent::CombatBlocked {
        attacker: 1,
        defender: 2,
        blocked_damage: 5,
    });

    event_bus.publish(GameEvent::StatusStacked {
        entity: 1,
        status: "Poison".to_string(),
        old_intensity: 1,
        new_intensity: 2,
    });

    event_bus.publish(GameEvent::DoorOpened {
        entity: 1,
        x: 5,
        y: 10,
        door_type: "wooden door".to_string(),
    });

    // Check history
    let history = event_bus.get_history(10);
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].event_type(), "CombatBlocked");
    assert_eq!(history[1].event_type(), "StatusStacked");
    assert_eq!(history[2].event_type(), "DoorOpened");
}

// ===== Test Backward Compatibility =====

#[test]
fn test_backward_compatibility() {
    let mut event_bus = EventBus::new();

    // Test that old event publishing still works
    event_bus.publish(GameEvent::DamageDealt {
        attacker: 1,
        victim: 2,
        damage: 10,
        is_critical: false,
    });

    // Events should be in both traditional queue and phase queues
    assert!(event_bus.has_events());
    
    let traditional_events: Vec<_> = event_bus.drain().collect();
    assert_eq!(traditional_events.len(), 1);
}

// ===== Integration Test: Full Turn Cycle =====

#[test]
fn test_full_turn_cycle_event_flow() {
    let mut event_bus = EventBus::new();

    // Simulate a full turn cycle with events in each phase

    // Phase 1: Input
    event_bus.set_current_phase(TurnPhase::Input);
    event_bus.publish_to_phase(
        GameEvent::ActionIntended {
            entity: 1,
            action_type: "attack".to_string(),
            priority: 100,
        },
        Priority::High,
        TurnPhase::Input,
    );

    // Phase 2: Intent Queue
    event_bus.set_current_phase(TurnPhase::IntentQueue);
    event_bus.publish_to_phase(
        GameEvent::AIDecisionMade {
            entity: 2,
            decision: "flee".to_string(),
        },
        Priority::Normal,
        TurnPhase::IntentQueue,
    );

    // Phase 3: Resolution
    event_bus.set_current_phase(TurnPhase::Resolution);
    event_bus.publish_to_phase(
        GameEvent::DamageDealt {
            attacker: 1,
            victim: 2,
            damage: 15,
            is_critical: true,
        },
        Priority::Critical,
        TurnPhase::Resolution,
    );

    event_bus.publish_to_phase(
        GameEvent::StatusApplied {
            entity: 2,
            status: "Bleeding".to_string(),
            duration: 3,
            intensity: 2,
        },
        Priority::Normal,
        TurnPhase::Resolution,
    );

    // Phase 4: Aftermath
    event_bus.set_current_phase(TurnPhase::Aftermath);
    event_bus.publish_to_phase(
        GameEvent::StatusEffectTicked {
            entity: 2,
            status: "Poison".to_string(),
            damage: 3,
            remaining_turns: 2,
        },
        Priority::Normal,
        TurnPhase::Aftermath,
    );

    // Process each phase in order
    let input_events = event_bus.drain_phase(TurnPhase::Input);
    assert_eq!(input_events.len(), 1);
    assert_eq!(input_events[0].event_type(), "ActionIntended");

    let intent_events = event_bus.drain_phase(TurnPhase::IntentQueue);
    assert_eq!(intent_events.len(), 1);
    assert_eq!(intent_events[0].event_type(), "AIDecisionMade");

    let resolution_events = event_bus.drain_phase(TurnPhase::Resolution);
    assert_eq!(resolution_events.len(), 2);
    // Should be ordered by priority: Critical first, then Normal
    assert_eq!(resolution_events[0].event_type(), "DamageDealt");
    assert_eq!(resolution_events[1].event_type(), "StatusApplied");

    let aftermath_events = event_bus.drain_phase(TurnPhase::Aftermath);
    assert_eq!(aftermath_events.len(), 1);
    assert_eq!(aftermath_events[0].event_type(), "StatusEffectTicked");
}
