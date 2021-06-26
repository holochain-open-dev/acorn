use crate::{
  error::Error,
  validate_helpers::entry_from_element_create_or_update,
};
use dna_help::WrappedAgentPubKey;
use hdk::prelude::*;
use std::fmt;

// A Goal Card. This is a card on the SoA Tree which can be small or non-small, complete or
// incomplete, certain or uncertain, and contains text content.
// user hash and unix timestamp are included to prevent hash collisions.
#[hdk_entry(id = "goal")]
#[derive(Clone, PartialEq)]
pub struct Goal {
  pub content: String,
  pub user_hash: WrappedAgentPubKey,
  pub user_edit_hash: Option<WrappedAgentPubKey>,
  pub timestamp_created: f64,
  pub timestamp_updated: Option<f64>,
  pub hierarchy: Hierarchy,
  pub status: Status,
  pub tags: Option<Vec<String>>,
  pub description: String,
  pub time_frame: Option<TimeFrame>,
  pub is_imported: bool,
}

// can be updated
impl TryFrom<&Element> for Goal {
  type Error = Error;
  fn try_from(element: &Element) -> Result<Self, Self::Error> {
    entry_from_element_create_or_update::<Goal>(element)
  }
}

impl Goal {
  pub fn new(
    content: String,
    user_hash: WrappedAgentPubKey,
    user_edit_hash: Option<WrappedAgentPubKey>,
    timestamp_created: f64,
    timestamp_updated: Option<f64>,
    hierarchy: Hierarchy,
    status: Status,
    tags: Option<Vec<String>>,
    description: String,
    time_frame: Option<TimeFrame>,
    is_imported: bool,
  ) -> Self {
    Self {
      content,
      user_hash,
      user_edit_hash,
      timestamp_created,
      timestamp_updated,
      hierarchy,
      status,
      tags,
      description,
      time_frame,
      is_imported,
    }
  }
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
pub struct UIEnum(String);

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum Status {
  Uncertain,
  Incomplete,
  InProcess,
  Complete,
  InReview,
}

impl From<UIEnum> for Status {
  fn from(ui_enum: UIEnum) -> Self {
    match ui_enum.0.as_str() {
      "Incomplete" => Self::Incomplete,
      "InProcess" => Self::InProcess,
      "Complete" => Self::Complete,
      "InReview" => Self::InReview,
      _ => Self::Uncertain,
    }
  }
}
impl From<Status> for UIEnum {
  fn from(status: Status) -> Self {
    Self(status.to_string())
  }
}
impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum Hierarchy {
  Root,
  Trunk,
  Branch,
  Leaf,
  NoHierarchy,
}
impl From<UIEnum> for Hierarchy {
  fn from(ui_enum: UIEnum) -> Self {
    match ui_enum.0.as_str() {
      "Root" => Self::Root,
      "Trunk" => Self::Trunk,
      "Branch" => Self::Branch,
      "Leaf" => Self::Leaf,
      _ => Self::NoHierarchy,
    }
  }
}
impl From<Hierarchy> for UIEnum {
  fn from(hierarchy: Hierarchy) -> Self {
    Self(hierarchy.to_string())
  }
}
impl fmt::Display for Hierarchy {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone, PartialEq)]
pub struct TimeFrame {
  from_date: f64,
  to_date: f64,
}

impl TimeFrame {
  pub fn new(from_date: f64, to_date: f64) -> Self {
    Self { from_date, to_date }
  }
}