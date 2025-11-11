//! Graphics rendering for the Battle Simulator
//!
//! This crate provides:
//! - GPU-accelerated rendering using wgpu
//! - UI system using egui
//! - Camera controls
//! - Asset loading and management
//! - Visual effects (smoke, fire, etc.)

#[cfg(feature = "graphics")]
pub mod gpu;

#[cfg(feature = "graphics")]
pub mod ui;

#[cfg(feature = "graphics")]
pub mod camera;

#[cfg(feature = "graphics")]
pub mod assets;

pub mod headless;

use battle_sim_core::*;

/// Renderer configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
    pub msaa_samples: u32,
    pub max_fps: Option<u32>,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            vsync: true,
            msaa_samples: 4,
            max_fps: None,
        }
    }
}
