# Adding a New Module

This guide explains how to add a new packet manipulation module to Myra.

## Overview

With the module registry pattern, adding a new module requires changes in **5-6 files** instead of 10+:

| File | What to add |
|------|-------------|
| `settings/<module>.rs` | Options struct |
| `settings/mod.rs` | `pub mod <module>` |
| `settings/manipulation.rs` | Field in `Settings` |
| `network/modules/<module>.rs` | Module implementation |
| `network/modules/mod.rs` | Export the module |
| `network/modules/registry.rs` | Register + wire up processing |

Stats are optional - only add if you need module-specific statistics.

---

## Step-by-Step: Adding a "Jitter" Module

### 1. Create the Options (`settings/jitter.rs`)

```rust
use crate::network::types::probability::Probability;
use crate::settings::default_true;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
pub struct JitterOptions {
    #[arg(skip)]
    #[serde(default)]
    pub enabled: bool,

    #[arg(skip)]
    #[serde(default = "default_true")]
    pub inbound: bool,

    #[arg(skip)]
    #[serde(default = "default_true")]
    pub outbound: bool,

    #[arg(skip)]
    #[serde(default)]
    pub probability: Probability,

    #[arg(skip)]
    #[serde(default)]
    pub duration_ms: u64,

    // === Module-specific fields below ===
    
    #[serde(default = "default_max_jitter")]
    pub max_jitter_ms: u64,

    #[serde(default)]
    pub min_jitter_ms: u64,
}

fn default_max_jitter() -> u64 { 50 }

impl Default for JitterOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            inbound: true,
            outbound: true,
            probability: Probability::default(),
            duration_ms: 0,
            max_jitter_ms: 50,
            min_jitter_ms: 0,
        }
    }
}

impl crate::network::modules::ModuleOptions for JitterOptions {
    fn is_enabled(&self) -> bool { self.enabled }
}
```

### 2. Register in Settings (`settings/mod.rs`)

```rust
pub mod jitter;  // Add this line
```

### 3. Add to Settings Struct (`settings/manipulation.rs`)

```rust
use crate::settings::jitter::JitterOptions;  // Add import

pub struct Settings {
    // ... existing fields ...
    
    #[serde(serialize_with = "serialize_option")]
    pub jitter: Option<JitterOptions>,  // Add field
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // ... existing ...
            jitter: None,
        }
    }
}
```

### 4. Create the Module (`network/modules/jitter.rs`)

```rust
use crate::error::Result;
use crate::network::core::PacketData;
use crate::network::modules::traits::{ModuleContext, PacketModule};
use crate::settings::jitter::JitterOptions;

#[derive(Debug, Default)]
pub struct JitterModule;

impl PacketModule for JitterModule {
    type Options = JitterOptions;
    type State = ();  // Use () for stateless modules

    fn name(&self) -> &'static str { "jitter" }
    fn display_name(&self) -> &'static str { "Packet Jitter" }
    fn get_duration_ms(&self, options: &Self::Options) -> u64 { options.duration_ms }

    fn process(
        &self,
        packets: &mut Vec<PacketData<'_>>,
        options: &Self::Options,
        _state: &mut Self::State,
        _ctx: &mut ModuleContext,
    ) -> Result<()> {
        // Your jitter implementation here
        Ok(())
    }
}
```

### 5. Export in `network/modules/mod.rs`

```rust
pub mod jitter;
pub use jitter::JitterModule;
```

### 6. Register in Registry (`network/modules/registry.rs`)

**Add to `MODULES` const:**
```rust
ModuleEntry {
    name: "jitter",
    display_name: "Packet Jitter", 
    order: 25,  // Between lag (20) and throttle (30)
    needs_special_handling: false,
},
```

**Add to `is_module_enabled()`:**
```rust
"jitter" => settings.jitter.as_ref().is_some_and(|o| o.enabled),
```

**Add to `process_all_modules()`:**
```rust
process_module(
    &JitterModule,
    settings.jitter.as_ref(),
    packets,
    &mut (),
    &mut state.effect_start_times.jitter,
    statistics,
    has_packets,
)?;
```

### 7. Add Effect Start Time (`network/processing/module_state.rs`)

```rust
pub struct ModuleEffectStartTimes {
    // ... existing fields ...
    pub jitter: Instant,
}

impl Default for ModuleEffectStartTimes {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            // ... existing ...
            jitter: now,
        }
    }
}
```

---

## Done! ✅

The registry pattern provides these utilities automatically:
- `is_module_enabled(settings, "jitter")` - Check if enabled
- `get_enabled_modules(settings)` - List all enabled modules  
- `has_any_enabled(settings)` - Check if anything is active
- `find_module("jitter")` - Get module metadata
- `module_names()` - Iterate all module names

---

## Optional Additions

### Builder Methods (`settings/builder.rs`)

```rust
pub fn jitter(mut self, max_ms: u64) -> Self {
    self.settings.jitter = Some(JitterOptions {
        enabled: true,
        max_jitter_ms: max_ms,
        ..Default::default()
    });
    self
}
```

### Module-Specific Statistics

Only if needed, create `network/modules/stats/jitter_stats.rs` and add to `PacketProcessingStatistics`.

### Stateful Modules

If your module needs persistent state between calls:

1. Define a state struct: `pub struct JitterState { ... }`
2. Add to `ModuleProcessingState` in `module_state.rs`
3. Pass `&mut state.jitter` instead of `&mut ()` in registry
