# Test Suite Activation Guide

This guide provides step-by-step instructions to activate the comprehensive test suite.

## Quick Activation

```bash
# 1. Rename .todo files back to .rs
cd tests
for f in *.todo; do mv "$f" "${f%.todo}"; done

cd ../benches
mv turn_pipeline_bench.rs.todo turn_pipeline_bench.rs

cd ..

# 2. Uncomment benchmark configuration in Cargo.toml
sed -i 's/# \[\[bench\]\]/[[bench]]/' Cargo.toml
sed -i 's/# name = "turn_pipeline_bench"/name = "turn_pipeline_bench"/' Cargo.toml
sed -i 's/# harness = false/harness = false/' Cargo.toml
```

## API Fixes Required

### 1. Hunger Component

**Current API:**
```rust
pub struct Hunger {
    pub satiety: u8,           // 0-10 (SPD standard)
    pub last_hunger_turn: u32,
}
```

**Find and replace in test files:**
```
hunger.current -> hunger.satiety
hunger.max -> 10  (constant max)
Hunger { current: ..., max: ... } -> Hunger { satiety: ..., last_hunger_turn: 0 }
```

### 2. ItemSlot Component

**Current API:**
```rust
pub struct ItemSlot {
    pub item: Option<ECSItem>,
    pub quantity: u32,
}
```

**Find and replace in test files:**
```
ItemSlot(n) -> ItemSlot { item: Some(item), quantity: 1 }
PlayerAction::UseItem(ItemSlot(0)) -> PlayerAction::UseItem(ItemSlot { item: Some(...), quantity: 1 })
```

### 3. Tile Component

**Current API:**
```rust
pub struct Tile {
    pub terrain_type: TerrainType,
    pub blocks_sight: bool,  // NOT blocks_vision!
    pub has_items: bool,
    pub has_monster: bool,
}
```

**Find and replace in test files:**
```
blocks_vision -> blocks_sight
is_passable -> (check terrain_type)
Tile { terrain, is_passable, blocks_vision } -> Tile { 
    terrain_type: terrain,
    blocks_sight: matches!(terrain, TerrainType::Wall),
    has_items: false,
    has_monster: false,
}
```

### 4. Stats Component

Current Stats structure includes:
- `hp`, `max_hp`
- `attack`, `defense`
- `accuracy`, `evasion`
- `strength`, `dexterity`, `intelligence`
- `level`, `experience`
- `class: Option<Class>`

No changes needed if using these fields correctly.

## Systematic Approach

### Step 1: Update test_helpers.rs (helpers/mod.rs)

```bash
cd tests/helpers
vim mod.rs  # or your preferred editor
```

Update the `TestWorldBuilder` methods:

1. **with_player**: Update Hunger initialization
2. **with_enemy**: Update Stats and Energy
3. **with_tile**: Update to use new Tile structure
4. **with_item**: Update ItemSlot usage

### Step 2: Compile and fix each test file individually

```bash
# Start with integration tests
cd ..
mv integration_tests.rs.todo integration_tests.rs
cargo test --test integration_tests 2>&1 | grep "error\[" | head -20

# Fix errors, then move to next
mv regression_tests.rs.todo regression_tests.rs
cargo test --test regression_tests 2>&1 | grep "error\[" | head -20

# Continue with others
mv performance_tests.rs.todo performance_tests.rs
mv save_load_tests.rs.todo save_load_tests.rs
mv property_tests.rs.todo property_tests.rs
```

### Step 3: Update benchmarks

```bash
cd ../benches
mv turn_pipeline_bench.rs.todo turn_pipeline_bench.rs
cargo bench --no-run 2>&1 | grep "error\[" | head -20
```

### Step 4: Enable benchmark in Cargo.toml

Edit `Cargo.toml`:
```toml
# Change from:
# [[bench]]
# name = "turn_pipeline_bench"
# harness = false

# To:
[[bench]]
name = "turn_pipeline_bench"
harness = false
```

## Testing the Activation

After each fix, verify:

```bash
# Compile check
cargo check --workspace --all-targets

# Run specific test
cargo test --test integration_tests

# Run all tests
cargo test --workspace

# Run benchmarks
cargo bench
```

## Common Errors and Solutions

### Error: "no field `current` on type `Hunger`"
**Solution:** Change to `satiety`

### Error: "expected struct, found tuple struct `ItemSlot`"
**Solution:** Use struct initialization: `ItemSlot { item: Some(...), quantity: 1 }`

### Error: "no field `blocks_vision` on type `Tile`"
**Solution:** Change to `blocks_sight`

### Error: "no field `is_passable` on type `Tile`"
**Solution:** Remove and check `terrain_type` directly

### Error: "no field `name` on type `ItemSlot`"
**Solution:** Access via `item_slot.item.as_ref().map(|i| &i.name)`

## Validation Checklist

- [ ] All test files renamed from `.todo` to `.rs`
- [ ] Benchmark file renamed and enabled in Cargo.toml
- [ ] `cargo check --workspace --all-targets` succeeds
- [ ] `cargo test --lib --workspace` passes
- [ ] `cargo test --test integration_tests` passes
- [ ] `cargo test --test regression_tests` passes
- [ ] `cargo test --test performance_tests --release` passes
- [ ] `cargo test --test save_load_tests` passes
- [ ] `cargo test --test property_tests` passes
- [ ] `cargo bench --no-run` succeeds
- [ ] CI workflow passes on GitHub

## Estimated Time

- Initial rename and setup: 15 minutes
- test_helpers.rs updates: 30-60 minutes
- integration_tests.rs fixes: 30-45 minutes
- regression_tests.rs fixes: 30-45 minutes
- performance_tests.rs fixes: 30-45 minutes
- save_load_tests.rs fixes: 20-30 minutes
- property_tests.rs fixes: 20-30 minutes
- Benchmark fixes: 20-30 minutes

**Total: 3-6 hours**

## Getting Help

If you encounter issues:

1. Check the current ECS API in `src/ecs.rs`
2. Look at working tests in `tests/adapters_eventbus_tests.rs`
3. Review existing lib tests for component usage patterns
4. Check `src/systems.rs` for real-world ECS component usage

## Once Complete

Update CI workflow to include performance tests:

```yaml
performance:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust
      uses: actions-rust-lang/setup-rust-toolchain@v1
      with:
        toolchain: stable
    - name: Run performance tests
      run: cargo test --test performance_tests --verbose --release
    - name: Run benchmarks (smoke test)
      run: cargo bench --no-run
```
