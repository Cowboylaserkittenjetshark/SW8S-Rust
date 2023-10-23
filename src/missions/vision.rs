use std::marker::PhantomData;

use super::action::{Action, ActionExec};
use crate::video_source::MatSource;
use crate::vision::{Offset2D, RelPos, VisualDetector};
use anyhow::Result;
use async_trait::async_trait;
use num_traits::Num;

/// Runs a vision routine to obtain object position
///
/// The relative position is normalized to [-1, 1] on both axes
#[derive(Debug)]
pub struct VisionNormOffset<T, U, V> {
    context: T,
    model: U,
    _num: PhantomData<V>,
}

impl<T, U, V> VisionNormOffset<T, U, V> {
    pub const fn new(context: T, model: U) -> Self {
        Self {
            context,
            model,
            _num: PhantomData,
        }
    }
}

impl<T, U, V> Action for VisionNormOffset<T, U, V> {}

#[async_trait]
impl<T: MatSource, V: Num + From<usize> + Send + Sync, U: VisualDetector<V> + Send + Sync>
    ActionExec<Result<Offset2D<V>>> for VisionNormOffset<T, U, V>
where
    U::Position: RelPos<Number = V>,
{
    async fn execute(mut self) -> Result<Offset2D<V>> {
        let detections = self.model.detect_unique(&self.context.get_mat().await)?;

        let positions: Vec<_> = detections
            .iter()
            .map(|detect| self.model.normalize(detect.position()))
            .map(|detect| detect.offset())
            .collect();

        let positions_len = positions.len();

        Ok(positions.into_iter().sum::<Offset2D<V>>() / positions_len)
    }
}
