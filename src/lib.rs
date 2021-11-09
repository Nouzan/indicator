//! Abstractions for stream aggregation, which we call "Indicator" s.
//!
//! This crate provides abstractions of different levels to build indicators:
//!
//! - Indicators are combinations of some [`Operator`]s.
//!
//! - To handle [Time series] better, we introduced [`TickedOperator`] operators,
//! of which the input and output are both [`Tickable`] (items of a time series)
//! and sharing the same [`Tick`]. We provide many ticked version utils. For example,
//! you can apply different [`TickedOperator`]s (using [`ticked::Facet`] or [`ticked::FacetMap`]) to the same [`Tickable`] stream
//! to get a "synced" result stream of the combined outputs of those operators
//! sharing the same [`Tick`] of theirs input.
//! Each item of a "synced" stream will have the form of `TickValue<(O1, O2)>`.
//!
//! - We can use [`WindowedOperation`] to create the corrresponding [`TickedOperator`].
//! For now, only [`Tumbling`] operations are supported, which are the operations defined on [`TumblingWindow`].
//!
//! [Time series]: https://en.wikipedia.org/wiki/Time_series

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

/// Operator.
pub mod operator;

/// Time window.
pub mod window;

/// Ticked operators.
pub mod ticked;

pub use operator::{IntoOperator, Operator, OperatorExt};
pub use ticked::{
    tumbling::{QueueCapAtLeast, Tumbling, TumblingOperator, TumblingQueue},
    TickedOperator,
};
pub use window::{Period, Tick, TickValue, Tickable, TumblingWindow, WindowMode};
