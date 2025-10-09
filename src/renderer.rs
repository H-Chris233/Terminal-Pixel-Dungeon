//! Ratatui renderer implementation for the ECS architecture.

use crate::ecs::*;

use ratatui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color as TuiColor, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Widget},
    Frame, Terminal,
};
use std::io::{self, Stdout};
use std::time::Duration;
use std::collections::HashMap;

/// Trait for rendering the game state
pub trait Renderer {
    type Backend: Backend;
    
    /// Initialize the renderer
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Draw the current game state
    fn draw(&mut self, ecs_world: &mut ECSWorld) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Draw UI elements
    fn draw_ui(&mut self, frame: &mut Frame<'_>, area: Rect);
    
    /// Cleanup resources
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>>;
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

/// Ratatui terminal renderer implementation
pub struct RatatuiRenderer {
    terminal: Terminal<ratatui::backend::CrosstermBackend<Stdout>>,
    last_render_time: std::time::Instant,
    render_cache: HashMap<(i32, i32, i32), RenderCacheEntry>, // x, y, z coordinates
}

/// Cached rendering data for optimization
struct RenderCacheEntry {
    symbol: char,
    fg: TuiColor,
    bg: TuiColor,
    timestamp: std::time::Instant,
}

impl RatatuiRenderer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            last_render_time: std::time::Instant::now(),
            render_cache: HashMap::new(),
        })
    }
    
    /// Render the ECS world to the terminal
    fn render_ecs_world(&mut self, ecs_world: &mut ECSWorld) -> Result<(), Box<dyn std::error::Error>> {
        self.terminal.draw(|f| {
            self.render_frame(f, ecs_world);
        })?;
        Ok(())
    }
    
    /// Render a frame with ECS world data
    fn render_frame(&self, frame: &mut Frame<'_>, ecs_world: &ECSWorld) {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Status bar
                Constraint::Min(10),     // Main game area
                Constraint::Length(3),   // Message log
            ])
            .split(frame.size());
        
        // Draw status bar
        let status_bar = Paragraph::new("Pixel Dungeon - Status")
            .style(Style::default().fg(TuiColor::Yellow))
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(status_bar, chunks[0]);
        
        // Draw main game area
        let game_area = GameWidget {
            ecs_world,
            player_pos: find_player_position(ecs_world),
        };
        frame.render_widget(game_area, chunks[1]);
        
        // Draw message log
        let messages = Paragraph::new(format_messages(&ecs_world.resources.game_state.message_log))
            .style(Style::default().fg(TuiColor::Gray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(messages, chunks[2]);
    }
}

impl Renderer for RatatuiRenderer {
    type Backend = ratatui::backend::CrosstermBackend<Stdout>;
    
    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
    
    fn draw(&mut self, ecs_world: &mut ECSWorld) -> Result<(), Box<dyn std::error::Error>> {
        self.render_ecs_world(ecs_world)
    }
    
    fn draw_ui(&mut self, frame: &mut Frame<'_>, area: Rect) {
        // Draw UI elements in the provided area
        let block = Block::default()
            .title("UI Panel")
            .borders(Borders::ALL);
        frame.render_widget(block, area);
    }
    
    fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Widget to render the game world from ECS
struct GameWidget<'a> {
    ecs_world: &'a ECSWorld,
    player_pos: Option<Position>,
}

impl<'a> Widget for GameWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Fill the area with background
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y)
                    .set_char(' ')
                    .set_fg(TuiColor::Black)
                    .set_bg(TuiColor::Black);
            }
        }

        // Get player's viewshed to determine what to render
        let visible_positions = get_visible_positions(self.ecs_world);
        
        // Render tiles on the same level as player
        let current_level = self.player_pos.as_ref().map_or(0, |pos| pos.z);
        
        // Render tiles
        for (entity, (pos, tile, _renderable)) in self.ecs_world.world.query::<(&Position, &Tile, &Renderable)>().iter() {
            if pos.z != current_level {
                continue; // Only render tiles on the same level as player
            }

            // Only render visible tiles
            if !visible_positions.contains(&(pos.x, pos.y)) {
                // Render as dark if not visible but remembered
                continue; // For now, only show visible tiles
            }

            let x = area.left() + pos.x as u16;
            let y = area.top() + pos.y as u16;

            // Check bounds
            if x < area.right() && y < area.bottom() {
                let cell = buf.get_mut(x, y);
                
                // Set the tile's appearance
                cell.set_char(tile.terrain_type.to_char());
                
                // Convert game color to ratatui color
                match &tile.terrain_type {
                    TerrainType::Wall => {
                        cell.set_fg(TuiColor::Gray).set_bg(TuiColor::DarkGray);
                    }
                    TerrainType::Floor => {
                        cell.set_fg(TuiColor::White).set_bg(TuiColor::Black);
                    }
                    TerrainType::Door => {
                        cell.set_fg(TuiColor::Yellow).set_bg(TuiColor::Black);
                    }
                    TerrainType::StairsDown => {
                        cell.set_fg(TuiColor::Cyan).set_bg(TuiColor::Black);
                    }
                    TerrainType::StairsUp => {
                        cell.set_fg(TuiColor::Blue).set_bg(TuiColor::Black);
                    }
                    TerrainType::Water => {
                        cell.set_fg(TuiColor::Blue).set_bg(TuiColor::Blue);
                    }
                    TerrainType::Trap => {
                        cell.set_fg(TuiColor::Red).set_bg(TuiColor::Black);
                    }
                    TerrainType::Barrel => {
                        cell.set_fg(TuiColor::Yellow).set_bg(TuiColor::Black);
                    }
                    TerrainType::Empty => {
                        cell.set_fg(TuiColor::Black).set_bg(TuiColor::Black);
                    }
                }
            }
        }

        // Render entities (actors, items, etc.)
        for (entity, (pos, renderable, _actor)) in self.ecs_world.world.query::<(&Position, &Renderable, &Actor)>().iter() {
            if pos.z != current_level {
                continue; // Only render entities on the same level as player
            }

            // Only render visible entities
            if !visible_positions.contains(&(pos.x, pos.y)) {
                continue;
            }

            let x = area.left() + pos.x as u16;
            let y = area.top() + pos.y as u16;

            // Check bounds
            if x < area.right() && y < area.bottom() {
                let cell = buf.get_mut(x, y);
                
                cell.set_char(renderable.symbol);
                
                // Convert game color to ratatui color
                cell.set_fg(renderable.fg_color.clone().into());
                
                if let Some(bg_color) = &renderable.bg_color {
                    cell.set_bg(bg_color.clone().into());
                }
            }
        }
    }
}

/// Helper function to get player's position
fn find_player_position(ecs_world: &ECSWorld) -> Option<Position> {
    for (entity, (pos, _actor)) in ecs_world.world.query::<(&Position, &Actor)>().iter() {
        // In a real implementation, we'd check if this is the player specifically
        // For now, we'll just return the first actor as the player
        if ecs_world.world.contains(entity) && ecs_world.world.get::<Player>(entity).is_ok() {
            return Some(pos.clone());
        }
    }
    None
}

/// Helper function to get visible positions from player's viewshed
fn get_visible_positions(ecs_world: &ECSWorld) -> std::collections::HashSet<(i32, i32)> {
    let mut visible_positions = std::collections::HashSet::new();
    
    for (entity, (viewshed, _pos, _actor)) in ecs_world.world.query::<(&Viewshed, &Position, &Actor)>().iter() {
        if ecs_world.world.contains(entity) && ecs_world.world.get::<Player>(entity).is_ok() { // Only player's viewshed
            for pos in &viewshed.visible_tiles {
                visible_positions.insert((pos.x, pos.y));
            }
            break;
        }
    }
    
    visible_positions
}

/// Helper function to format messages for display
fn format_messages(messages: &[String]) -> String {
    if messages.is_empty() {
        "Welcome to Pixel Dungeon!".to_string()
    } else {
        messages.join("\n")
    }
}

/// Extension trait to convert terrain type to character representation
trait ToChar {
    fn to_char(&self) -> char;
}

impl ToChar for TerrainType {
    fn to_char(&self) -> char {
        match self {
            TerrainType::Wall => '#',
            TerrainType::Floor => '.',
            TerrainType::Door => '+',
            TerrainType::StairsDown => '>',
            TerrainType::StairsUp => '<',
            TerrainType::Water => '~',
            TerrainType::Trap => '^',
            TerrainType::Barrel => 'O',
            TerrainType::Empty => ' ',
        }
    }
}

/// Clock implementation for time management
pub struct GameClock {
    tick_rate: Duration,
    start_time: std::time::SystemTime,
}

impl GameClock {
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
            start_time: std::time::SystemTime::now(),
        }
    }
}

impl Clock for GameClock {
    fn now(&self) -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
    
    fn elapsed(&self, since: std::time::SystemTime) -> Duration {
        self.now().duration_since(since).unwrap_or(Duration::from_millis(0))
    }
    
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
    
    fn tick_rate(&self) -> Duration {
        self.tick_rate
    }
}

/// Convert ECS colors to ratatui colors
impl From<Color> for TuiColor {
    fn from(color: Color) -> Self {
        match color {
            Color::Red => TuiColor::Red,
            Color::Green => TuiColor::Green,
            Color::Yellow => TuiColor::Yellow,
            Color::Blue => TuiColor::Blue,
            Color::Magenta => TuiColor::Magenta,
            Color::Cyan => TuiColor::Cyan,
            Color::Gray => TuiColor::Gray,
            Color::DarkGray => TuiColor::DarkGray,
            Color::White => TuiColor::White,
            Color::Black => TuiColor::Black,
            Color::Reset => TuiColor::Reset,
            Color::Rgb(r, g, b) => TuiColor::Rgb(r, g, b),
        }
    }
}