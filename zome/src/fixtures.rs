#[cfg(test)]
pub(crate) mod fixtures {
    use crate::goal::crud::{Goal, Hierarchy, Status};
    use crate::{goal::crud::TimeFrame, goal_comment::crud::GoalComment};
    use ::fixt::prelude::*;
    use dna_help::{WrappedAgentPubKey, WrappedHeaderHash};
    use hdk::prelude::*;

    // why can't I put this one in dna_help crate?
    fixturator!(
      WrappedHeaderHash;
        constructor fn new(HeaderHash);
    );

    fixturator!(
      WrappedAgentPubKey;
        constructor fn new(AgentPubKey);
    );

    fixturator!(
      GoalComment;
        constructor fn new(WrappedHeaderHash, String, WrappedAgentPubKey, f64, bool);
    );

    type OptionWrappedAgentPubKey = Option<WrappedAgentPubKey>;
    type OptionString = Option<String>;
    type Optionf64 = Option<f64>;
    type OptionVecString = Option<Vec<String>>;
    type OptionTimeFrame = Option<TimeFrame>;

    fixturator!(
      TimeFrame;
      constructor fn new(f64, f64);
    );

    fixturator!(
      Status;
      unit variants [ Uncertain Incomplete InProcess Complete InReview ] empty Uncertain;
    );

    fixturator!(
      Hierarchy;
      unit variants [Root Trunk Branch Leaf NoHierarchy ] empty NoHierarchy;
    );

    fixturator!(
      OptionWrappedAgentPubKey;
      curve Empty {
          None
      };
      curve Unpredictable {
        Some(WrappedAgentPubKeyFixturator::new(Unpredictable).next().unwrap())
      };
      curve Predictable {
        Some(WrappedAgentPubKeyFixturator::new(Predictable).next().unwrap())
      };
    );

    fixturator!(
      OptionString;
      curve Empty {
          None
      };
      curve Unpredictable {
        Some(fixt!(String))
      };
      curve Predictable {
        Some(fixt!(String, Predictable))
      };
    );

    fixturator!(
      Optionf64;
      curve Empty {
          None
      };
      curve Unpredictable {
        Some(fixt!(f64))
      };
      curve Predictable {
        Some(fixt!(f64, Predictable))
      };
    );

    fixturator!(
      OptionVecString;
      curve Empty {
          None
      };
      curve Unpredictable {
        Some(Vec::new())
      };
      curve Predictable {
        Some(Vec::new())
      };
    );

    fixturator!(
      OptionTimeFrame;
      curve Empty {
          None
      };
      curve Unpredictable {
        Some(TimeFrameFixturator::new(Unpredictable).next().unwrap())
      };
      curve Predictable {
        Some(TimeFrameFixturator::new(Predictable).next().unwrap())
      };
    );

    fixturator!(
      Goal;
        constructor fn new(String, WrappedAgentPubKey, OptionWrappedAgentPubKey, f64, Optionf64, Hierarchy, Status, OptionVecString, String, OptionTimeFrame, bool);
    );
}
