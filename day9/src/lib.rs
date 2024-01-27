//! Word analysis library for Day 9 exercises.
//!
//! # Examples
//! ```rust
//! use day9::analyze_text;
//!
//! let result = analyze_text("Rust; rust: RUST");
//! assert_eq!(result.total_words, 3);
//! assert_eq!(result.unique_words, 1);
//! ```
//!
//! ```rust
//! use day9::analyze_text;
//!
//! let result = analyze_text("");
//! assert_eq!(result.total_words, 0);
//! assert_eq!(result.unique_words, 0);
//! ```
mod analysis;
mod normalize;

pub use analysis::{AnalysisResult, analyze_text};
