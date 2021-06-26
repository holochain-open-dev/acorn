use hdk::prelude::*;

mod goal;
mod goal_comment;
mod fixtures;
mod error;
mod validate_helpers;

use goal::crud::Goal;
use goal_comment::crud::GoalComment;

entry_defs!(
    Path::entry_def(),
    Goal::entry_def(),
    GoalComment::entry_def()
);
