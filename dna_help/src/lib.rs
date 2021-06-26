use hdk::prelude::*;
pub use paste;
use std::fmt;
use thiserror::Error;

#[cfg(test)]
use ::fixt::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, SerializedBytes)]
#[serde(from = "UIEnum")]
#[serde(into = "UIEnum")]
pub enum ActionType {
    Create,
    Update,
    Delete,
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
pub struct UIEnum(String);

impl From<UIEnum> for ActionType {
    fn from(ui_enum: UIEnum) -> Self {
        match ui_enum.0.as_str() {
            "create" => Self::Create,
            "update" => Self::Update,
            _ => Self::Delete,
        }
    }
}

impl From<ActionType> for UIEnum {
    fn from(action_type: ActionType) -> Self {
        Self(action_type.to_string().to_lowercase())
    }
}

impl fmt::Display for ActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type EntryAndHash<T> = (T, HeaderHash, EntryHash);
pub type OptionEntryAndHash<T> = Option<EntryAndHash<T>>;

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
pub struct UIStringHash(String);

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
#[serde(try_from = "UIStringHash")]
#[serde(into = "UIStringHash")]
pub struct WrappedAgentPubKey(pub AgentPubKey);

impl WrappedAgentPubKey {
  pub fn new(agent_pubkey: AgentPubKey) -> Self {
      Self(agent_pubkey)
  }
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
#[serde(try_from = "UIStringHash")]
#[serde(into = "UIStringHash")]
pub struct WrappedHeaderHash(pub HeaderHash);

impl WrappedHeaderHash {
  pub fn new(header_hash: HeaderHash) -> Self {
      Self(header_hash)
  }
}

#[derive(Debug, Serialize, Deserialize, SerializedBytes, Clone, PartialEq)]
#[serde(try_from = "UIStringHash")]
#[serde(into = "UIStringHash")]
pub struct WrappedEntryHash(pub EntryHash);

impl TryFrom<UIStringHash> for WrappedAgentPubKey {
    type Error = String;
    fn try_from(ui_string_hash: UIStringHash) -> Result<Self, Self::Error> {
        match AgentPubKey::try_from(ui_string_hash.0) {
            Ok(address) => Ok(Self(address)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
impl From<WrappedAgentPubKey> for UIStringHash {
    fn from(wrapped_agent_pub_key: WrappedAgentPubKey) -> Self {
        Self(wrapped_agent_pub_key.0.to_string())
    }
}

impl TryFrom<UIStringHash> for WrappedHeaderHash {
    type Error = String;
    fn try_from(ui_string_hash: UIStringHash) -> Result<Self, Self::Error> {
        match HeaderHash::try_from(ui_string_hash.0) {
            Ok(address) => Ok(Self(address)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
impl From<WrappedHeaderHash> for UIStringHash {
    fn from(wrapped_header_hash: WrappedHeaderHash) -> Self {
        Self(wrapped_header_hash.0.to_string())
    }
}

impl TryFrom<UIStringHash> for WrappedEntryHash {
    type Error = String;
    fn try_from(ui_string_hash: UIStringHash) -> Result<Self, Self::Error> {
        match EntryHash::try_from(ui_string_hash.0) {
            Ok(address) => Ok(Self(address)),
            Err(e) => Err(format!("{:?}", e)),
        }
    }
}
impl From<WrappedEntryHash> for UIStringHash {
    fn from(wrapped_entry_hash: WrappedEntryHash) -> Self {
        Self(wrapped_entry_hash.0.to_string())
    }
}

/*
  SIGNALS
*/

pub fn create_receive_signal_cap_grant() -> ExternResult<()> {
    let mut functions: GrantedFunctions = BTreeSet::new();
    functions.insert((zome_info()?.zome_name, "recv_remote_signal".into()));

    create_cap_grant(CapGrantEntry {
        tag: "".into(),
        // empty access converts to unrestricted
        access: ().into(),
        functions,
    })?;
    Ok(())
}

pub fn get_header_hash(shh: element::SignedHeaderHashed) -> HeaderHash {
    shh.header_hashed().as_hash().to_owned()
}

pub fn get_latest_for_entry<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    entry_hash: EntryHash,
    get_options: GetOptions,
) -> ExternResult<OptionEntryAndHash<T>> {
    // First, make sure we DO have the latest header_hash address
    let maybe_latest_header_hash = match get_details(entry_hash.clone(), get_options.clone())? {
        Some(Details::Entry(details)) => match details.entry_dht_status {
            metadata::EntryDhtStatus::Live => match details.updates.len() {
                // pass out the header associated with this entry
                0 => Some(get_header_hash(details.headers.first().unwrap().to_owned())),
                _ => {
                    let mut sortlist = details.updates.to_vec();
                    // unix timestamp should work for sorting
                    sortlist.sort_by_key(|update| update.header().timestamp().0);
                    // sorts in ascending order, so take the last element
                    let last = sortlist.last().unwrap().to_owned();
                    Some(get_header_hash(last))
                }
            },
            metadata::EntryDhtStatus::Dead => None,
            _ => None,
        },
        _ => None,
    };

    // Second, go and get that element, and return it and its header_address
    match maybe_latest_header_hash {
        Some(latest_header_hash) => match get(latest_header_hash, get_options)? {
            Some(element) => match element.entry().to_app_option::<T>()? {
                Some(entry) => Ok(Some((
                    entry,
                    match element.header() {
                        // we DO want to return the header for the original
                        // instead of the updated, in our case
                        Header::Update(update) => update.original_header_address.clone(),
                        Header::Create(_) => element.header_address().clone(),
                        _ => unreachable!("Can't have returned a header for a nonexistent entry"),
                    },
                    element.header().entry_hash().unwrap().to_owned(),
                ))),
                None => Ok(None),
            },
            None => Ok(None),
        },
        None => Ok(None),
    }
}

pub fn fetch_links<
    EntryType: TryFrom<SerializedBytes, Error = SerializedBytesError>,
    WireEntry: From<EntryAndHash<EntryType>>,
>(
    entry_hash: EntryHash,
    get_options: GetOptions,
) -> Result<Vec<WireEntry>, WasmError> {
    Ok(get_links(entry_hash, None)?
        .into_inner()
        .into_iter()
        .map(|link: link::Link| {
            get_latest_for_entry::<EntryType>(link.target.clone(), get_options.clone())
        })
        .filter_map(Result::ok)
        .filter_map(|x| x)
        .map(|x| WireEntry::from(x))
        .collect())
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Element missing its Entry")]
    EntryMissing,
}

impl From<Error> for ValidateCallbackResult {
    fn from(e: Error) -> Self {
        ValidateCallbackResult::Invalid(e.to_string())
    }
}

impl From<Error> for ExternResult<ValidateCallbackResult> {
    fn from(e: Error) -> Self {
        Ok(e.into())
    }
}

pub struct ResolvedDependency<D>(pub Element, pub D);

pub fn resolve_dependency<'a, O>(
    hash: AnyDhtHash,
) -> ExternResult<Result<ResolvedDependency<O>, ValidateCallbackResult>>
where
    O: TryFrom<SerializedBytes, Error = SerializedBytesError>,
{
    let element = match get(hash.clone(), GetOptions::content())? {
        Some(element) => element,
        None => {
            return Ok(Err(ValidateCallbackResult::UnresolvedDependencies(vec![
                hash,
            ])))
        }
    };

    let output: O = match element.entry().to_app_option() {
        Ok(Some(output)) => output,
        Ok(None) => return Ok(Err(Error::EntryMissing.into())),
        Err(e) => return Ok(Err(ValidateCallbackResult::Invalid(e.to_string()))),
    };

    Ok(Ok(ResolvedDependency(element, output)))
}