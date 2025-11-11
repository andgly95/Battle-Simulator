//! Rendering system for the Battle Simulator
//!
//! This crate provides a clean abstraction layer for rendering, supporting
//! multiple backends from simple 2D pixels to full 3D animated scenes.
//!
//! # Architecture
//!
//! - **Phase 1 (Current)**: 2D pixel-based rendering using `pixels` crate
//! - **Phase 2 (Future)**: 2D sprite-based rendering using `wgpu`
//! - **Phase 3 (Future)**: Full 3D rendering with animated models
//!
//! # Usage
//!
//! ```no_run
//! use battle_sim_renderer::PixelRenderer;
//! use winit::event_loop::EventLoop;
//! use winit::window::WindowBuilder;
//!
//! let event_loop = EventLoop::new();
//! let window = WindowBuilder::new()
//!     .with_title("Battle Simulator")
//!     .build(&event_loop)
//!     .unwrap();
//!
//! let mut renderer = PixelRenderer::new(&window).unwrap();
//!
//! // In event loop:
//! renderer.begin_frame();
//! // ... draw units ...
//! renderer.end_frame().unwrap();
//! ```

pub mod camera;
pub mod pixel_renderer;

// Re-exports for convenience
pub use camera::{Camera, Rect};
pub use pixel_renderer::{PixelRenderer, colors};
