//! Abstractions for stream aggregation that we call "Indicator" s.
//!
//! This crate provides abstractions of different levels to build indicators:
//!
//! - Indicators are the combinations of [`Operator`]s. We can chain them by [`OperatorExt::then`]
//! and apply them to the same input "simultaneously" by [`OperatorExt::facet`].
//!
//! - To handle [Time series] better, we introduced utils for ticked operators, which are defined in [`TickedOperatorExt`],
//! of which the input and output are both [`Tickable`] and sharing the same [`Tick`].
//! We provide many ticked version utils. For example, we can apply different [`TickedOperatorExt`]s
//! (using [`TickedOperatorExt::facet_t`] or [`ticked::FacetMap`]) to the same [`Tickable`] stream to
//! get a "synced" result stream of the combined outputs of those operators sharing the same [`Tick`]
//! of theirs input. Each item of a "synced" stream will have the form of `TickValue<(O1, O2)>`.
//!
//! - [`TumblingOperator`]s are defined by some operations on events of non-overlapping time windows ([`TumblingWindow`]).
//! [Moving average] is one of the famous examples, which is defined by the average of the numbers
//! from those events occur in the same tumbling window. We call those operations [`TumblingOperation`]s.
//! We can use [`tumbling`] function to create a [`TumblingOperator`] from a [`TumblingOperation`].
//!
//! - Finally, we can apply the indicators to [`Iterator`]s or [`Stream`](futures::stream::Stream)s
//! by using [`IndicatorIteratorExt::indicator`] or [`IndicatorStreamExt::indicator`] accordingly.
//!
//!
//! [Time series]: https://en.wikipedia.org/wiki/Time_series
//! [Moving average]: https://en.wikipedia.org/wiki/Moving_average
//!
//! # Example
//! ```
//! use indicator::*;
//! use rust_decimal::Decimal;
//! use rust_decimal_macros::dec;
//! use time::macros::offset;
//! use arrayvec::ArrayVec;
//!
//! /// Return an indicator that calculates `hl2` and `ohlc4` simultaneously.
//! fn hl2_ohlc4(period: Period) -> impl Operator<TickValue<Decimal>, Output = (Decimal, Decimal)> {
//!     tumbling(
//!         period,
//!         |_w: &ArrayVec<[Decimal; 4], 0>, y: &mut Option<[Decimal; 4]>, x| match y {
//!             Some(ohlc) => {
//!                 ohlc[1] = ohlc[1].max(x);
//!                 ohlc[2] = ohlc[2].min(x);
//!                 ohlc[3] = x;
//!                 *ohlc
//!             }
//!             None => {
//!                 let ohlc = [x; 4];
//!                 *y = Some(ohlc);
//!                 ohlc
//!             }
//!         },
//!     )
//!     .then(facet_t(
//!         map_t(|ohlc: [Decimal; 4]| (ohlc[1] + ohlc[2]) / dec!(2)),
//!         map_t(|ohlc: [Decimal; 4]| (ohlc[0] + ohlc[1] + ohlc[2] + ohlc[3]) / dec!(4)),
//!     ))
//!     .map(|v| v.value)
//! }
//!```

#![deny(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

/// Operator.
pub mod operator;

/// Time window.
pub mod window;

/// Ticked operators.
pub mod ticked;

/// Iterator extension trait.
pub mod iter;

#[cfg(feature = "stream")]
/// Stream extension trait.
pub mod stream;

/// Rayon supported combinator.
#[cfg(feature = "parallel")]
pub mod rayon;

/// Async operator support.
#[cfg(feature = "async")]
pub mod async_operator;

/// Operator using GAT.
#[cfg(feature = "gat")]
pub mod gat;

/// Prelude.
pub mod prelude {
    pub use crate::operator::{BoxOperator, LocalBoxOperator, Operator, OperatorExt};
    pub use crate::window::{Period, Tick, TickValue, TumblingWindow};

    #[cfg(feature = "gat")]
    pub use crate::gat::*;
}

pub use iter::IndicatorIteratorExt;
pub use operator::{facet, map, Operator, OperatorExt};
pub use ticked::{
    facet_t, map_t,
    tumbling::{
        cached, iterated, tumbling, Cached, CachedOperation, Iterated, IteratedOperation,
        QueueCapAtLeast, TumblingOperation, TumblingOperator, TumblingQueue,
    },
    tuple_t, TickedOperatorExt,
};
pub use window::{Period, PeriodKind, Tick, TickValue, Tickable, TumblingWindow};

#[cfg(feature = "std")]
pub use ticked::facet_map_t;

#[cfg(feature = "std")]
pub use facet::facet_map;

#[cfg(feature = "std")]
pub use ticked::{shared, SharedMap};

#[cfg(feature = "stream")]
pub use stream::IndicatorStreamExt;

#[cfg(feature = "array-vec")]
pub use ticked::array_t;

#[cfg(feature = "async")]
pub use async_operator::AsyncOperator;

#[cfg(feature = "tower")]
pub use async_operator::ServiceOperator;
