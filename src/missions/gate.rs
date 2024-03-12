use std::{iter::Sum, ops::Div};

use anyhow::{anyhow, bail};
use async_trait::async_trait;

use tokio::io::WriteHalf;
use tokio_serial::SerialStream;

use crate::{
    act_nest,
    vision::{
        gate_poles::{GatePoles, Target},
        nn_cv2::{OnnxModel, YoloClass},
        DrawRect2d, Offset2D, VisualDetection,
    },
};

use super::{
    action::{
        wrap_action, Action, ActionChain, ActionConcurrent, ActionExec, ActionMod, ActionSequence,
        ActionWhile, FirstValid, TupleSecond,
    },
    action_context::{GetControlBoard, GetFrontCamMat, GetMainElectronicsBoard},
    basic::{descend_and_go_forward, NoOp},
    comms::StartBno055,
    movement::{CountFalse, CountTrue},
    vision::{DetectTarget, VisionNorm},
};

pub fn gate_run_complex<
    Con: Send
        + Sync
        + GetControlBoard<WriteHalf<SerialStream>>
        + GetMainElectronicsBoard
        + GetFrontCamMat,
>(
    context: &Con,
) -> impl ActionExec + '_ {
    let depth: f32 = -1.0;

    ActionSequence::new(
        ActionConcurrent::new(descend_and_go_forward(context), StartBno055::new(context)),
        ActionSequence::new(
            adjust_logic(context, depth, CountTrue::new(3)),
            adjust_logic(context, depth, CountFalse::new(10)),
        ),
    )
}

pub fn adjust_logic<
    'a,
    Con: Send
        + Sync
        + GetControlBoard<WriteHalf<SerialStream>>
        + GetMainElectronicsBoard
        + GetFrontCamMat,
    X: 'a
        + ActionMod<anyhow::Result<Vec<VisualDetection<YoloClass<Target>, Offset2D<f64>>>>>
        + ActionExec<Output = anyhow::Result<()>>,
>(
    context: &'a Con,
    _depth: f32,
    end_condition: X,
) -> impl ActionExec + 'a {
    ActionWhile::new(ActionChain::new(
        VisionNorm::<Con, GatePoles<OnnxModel>, f64>::new(context, GatePoles::default()),
        TupleSecond::new(ActionConcurrent::new(
            act_nest!(
                wrap_action(ActionConcurrent::new, FirstValid::new),
                DetectTarget::<Target, YoloClass<Target>, Offset2D<f64>>::new(Target::Earth),
                DetectTarget::<Target, YoloClass<Target>, Offset2D<f64>>::new(Target::Abydos),
                DetectTarget::<Target, YoloClass<Target>, Offset2D<f64>>::new(Target::LargeGate),
            ),
            end_condition,
        )),
    ))
}
