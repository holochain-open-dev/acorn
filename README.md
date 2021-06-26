# How To Set Up Unit Testing

Requirements:

- Have rust language (stable) installed on your system

To test:

- run `./run-test.sh`

## Holochain Version In Use

branch: [2021-05-12-deepkey-tweaks](https://github.com/holochain/holochain/tree/2021-05-12-deepkey-tweaks)
date: May 12, 2021

Note that this branch has since been merged into the main branch, and so this repository could be updated to that latest at any point.

## Explanations

Because of new holochain developments, we are now able to start testing Zome functions and validation hooks without leaving Rust language, by using `cargo` to test, and a "Mock HDK" to mock (simulate) the holochain engine.

In order to utilize these mocking features, we need to wrap those features in a Rust feature of our own (to keep the code out of our shipped binaries and compile size small). We add the following to the `Cargo.toml` of our Zome:

`Cargo.toml`

```toml
[features]
mock = ["hdk/mock", "hdk/test_utils"]
```

Now, if we switch on the "mock" feature within our IDE or from the command line, we'll be able to run unit tests over our Zome functions and validation hooks with the Mock HDK.

This folder has a `.vscode` folder which stores a workspace preference to enable the `mock` feature within VSCode, if you're using the `rust-analyzer` extension which is highly recommended.

We can also see the passing of the "mock" feature during the custom `cargo test` command written in [run-test.sh](./run-test.sh): `--features="mock"`

Diving in to the code, examples of the unit tests are written into [zome/src/goal/validate.rs](zome/src/goal/validate.rs) and [zome/src/goal_comment/validate.rs](zome/src/goal_comment/validate.rs).

In those files, we test out the validation hooks that we define:

`zome/src/goal/validate.rs`

- validate_create_entry_goal
- validate_update_entry_goal
- validate_delete_entry_goal

`zome/src/goal_comment/validate.rs`

- validate_create_entry_goal_comment
- validate_update_entry_goal_comment
- validate_delete_entry_goal_comment

There is a unit test in the respective files for each of these functions.

You will see a few patterns in the code sample that will follow, here is a 1-liner on each of them:
- where you see `fixt!(ValidateData)` it is the concept of generating semi-random data quickly and easily based on a "testing fixture", which Holochain has a helper library crate for called [fixt](https://github.com/holochain/holochain/tree/2021-05-12-deepkey-tweaks/crates/fixt), which is the origin of this `fixt!` macro. 
- where you see `*validate_data.element.as_entry_mut() = ...`  it is using a rather memory unsafe function only available in testing mode in order to override the contents of some data located in memory, by accessing it mutably and "deferenced"/`*`.
- where you see `let mut mock_hdk = MockHdkT::new();` it is an example of generating a mockable holochain engine which matches entirely the function signatures of whichever version of the HDK you happen to be using at the time, whose calls can be mocked/simulated via patterns defined by the [mockall](https://crates.io/crates/mockall) library.
- where you see: `
mock_hdk.expect_get().with(mockall::predicate::eq(GetInput::new(
        goal_wrapped_header_hash.clone().0.into(),
        GetOptions::content(),
      ))).times(1).return_const(Ok(None));
`
  it is an example of mocking the `get` method of the HDK (pulled from `expect_get()`, for some other function like `hash_entry` it would be `expect_hash_entry`). It uses chaining `.this().that()` to define the intended response. See [mockall](https://crates.io/crates/mockall) crate for usage.
- when you see `set_hdk(mock_hdk);` it is making an update to *check this* a lazy static global variable which holds the current version of the HDK that function calls will refer to. 
- when you see `assert_eq!(super::validate_create_entry_goal_comment(validate_data.clone()), Ok(ValidateCallbackResult::Valid));` it is an example of making an assertion in your unit test that the validation hook being called by Holochain with a given expected input returns a given expected result.

Here is a single test from `zome/src/goal_comment/validate.rs`, with some additional code commenting:

```rust
// only compile this code if running in `test`
// configuration, standard rust/cargo style
#[cfg(test)]
pub mod tests {
  // import relevant types
  use crate::error::Error;
  use crate::fixtures::fixtures::{
    GoalCommentFixturator, GoalFixturator, WrappedAgentPubKeyFixturator,
    WrappedHeaderHashFixturator,
  };
  use ::fixt::prelude::*;
  use dna_help::WrappedAgentPubKey;
  use hdk::prelude::*;
  use holochain_types::prelude::ElementFixturator;
  use holochain_types::prelude::ValidateDataFixturator;

  // our test
  #[test]
  fn test_validate_create_entry_goal_comment() {
    let mut validate_data = fixt!(ValidateData);
    let create_header = fixt!(Create);
    let mut goal_comment = fixt!(GoalComment);
    // set is_imported to false so that we don't skip
    // important validation
    goal_comment.is_imported = false;

    // for the moment, we just update the dereferenced/raw
    // memory with the new data
    *validate_data.element.as_header_mut() = Header::Create(create_header.clone());

    // without an Element containing an Entry, validation will fail
    assert_eq!(
      super::validate_create_entry_goal_comment(validate_data.clone()),
      Error::EntryMissing.into(),
    );

    let goal_wrapped_header_hash = fixt!(WrappedHeaderHash);
    goal_comment.goal_address = goal_wrapped_header_hash.clone();
    *validate_data.element.as_entry_mut() =
      ElementEntry::Present(goal_comment.clone().try_into().unwrap());

    // now, since validation is dependent on other entries, we begin
    // to have to mock `get` calls to the HDK

    let mut mock_hdk = MockHdkT::new();
    // simulate the response to
    // the resolve_dependencies `get` call of the parent goal
    mock_hdk
      .expect_get()
      .with(mockall::predicate::eq(GetInput::new(
        goal_wrapped_header_hash.clone().0.into(),
        GetOptions::content(),
      )))
      .times(1)
      .return_const(Ok(None));

    set_hdk(mock_hdk);

    // we should see that the ValidateCallbackResult is that there are UnresolvedDependencies
    // equal to the Hash of the parent Goal address
    assert_eq!(
      super::validate_create_entry_goal_comment(validate_data.clone()),
      Ok(ValidateCallbackResult::UnresolvedDependencies(vec![
        goal_wrapped_header_hash.clone().0.into()
      ])),
    );

    // now make it as if there is a Goal at the parent_address
    // so that we pass the dependency validation
    let goal = fixt!(Goal);
    let mut goal_element = fixt!(Element);
    *goal_element.as_entry_mut() = ElementEntry::Present(goal.clone().try_into().unwrap());

    let mut mock_hdk = MockHdkT::new();
    // simulate the response to
    // the resolve_dependencies `get` call of the goal_address
    mock_hdk
      .expect_get()
      .with(mockall::predicate::eq(GetInput::new(
        goal_wrapped_header_hash.clone().0.into(),
        GetOptions::content(),
      )))
      .times(1)
      .return_const(Ok(Some(goal_element.clone())));

    set_hdk(mock_hdk);

    // with an entry with a random
    // agent_address it will fail (not the agent committing)
    let random_wrapped_agent_pub_key = fixt!(WrappedAgentPubKey);
    goal_comment.agent_address = random_wrapped_agent_pub_key.clone();
    *validate_data.element.as_entry_mut() =
      ElementEntry::Present(goal_comment.clone().try_into().unwrap());
    assert_eq!(
      super::validate_create_entry_goal_comment(validate_data.clone()),
      Error::CorruptCreateAgentPubKeyReference.into(),
    );

    // SUCCESS case
    // the element exists
    // the parent goal is found/exists
    // agent_address refers to the agent committing
    // -> good to go

    // make the agent_address valid by making it equal the
    // AgentPubKey of the agent committing
    goal_comment.agent_address = WrappedAgentPubKey::new(create_header.author.as_hash().clone());
    *validate_data.element.as_entry_mut() =
      ElementEntry::Present(goal_comment.clone().try_into().unwrap());

    // it is as if there is a Goal at the parent_address
    let goal = fixt!(Goal);
    let mut goal_element = fixt!(Element);
    *goal_element.as_entry_mut() = ElementEntry::Present(goal.clone().try_into().unwrap());

    let mut mock_hdk = MockHdkT::new();
    // simulate the response to 
    // the resolve_dependencies `get` call of the goal_address
    mock_hdk
      .expect_get()
      .with(mockall::predicate::eq(GetInput::new(
        goal_wrapped_header_hash.clone().0.into(),
        GetOptions::content(),
      )))
      .times(1)
      .return_const(Ok(Some(goal_element.clone())));

    set_hdk(mock_hdk);

    // we should see that the ValidateCallbackResult
    // is finally valid
    assert_eq!(
      super::validate_create_entry_goal_comment(validate_data.clone()),
      Ok(ValidateCallbackResult::Valid),
    );
  }
}
```
