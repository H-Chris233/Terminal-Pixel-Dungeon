//! Graphics and rendering abstractions for the ECS architecture.

use std::time::Duration;

use ratatui::backend::Backend;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::dungeon::Dungeon;
use crate::hero::Hero;

/// Trait for rendering the game state
pub trait Renderer {
    type Backend: Backend;
    
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Draw the current game state
    fn draw(&mut self, dungeon: &Dungeon, hero: &Hero) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Draw UI elements
    fn draw_ui(&mut self, frame: &mut Frame<Self::Backend>, area: Rect);
    
    /// Cleanup resources
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// Trait for updating the game state based on input
pub trait InputSource {
    type Event;
    
    /// Poll for input events
    fn poll(&mut self, timeout: Duration) -> Result<Option<Self::Event>, Box<dyn std::error::Error>>;
    
    /// Check if input is available
    fn is_input_available(&self) -> Result<bool, Box<dyn std::error::Error>>;
}

/// Trait for time management
pub trait Clock {
    /// Get the current time
    fn now(&self) -> std::time::SystemTime;
    
    /// Get elapsed time since a given point
    fn elapsed(&self, since: std::time::SystemTime) -> Duration;
    
    /// Sleep for duration
    fn sleep(&self, duration: Duration);
    
    /// Get fixed time step for game logic updates
    fn tick_rate(&self) -> Duration;
}