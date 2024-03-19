mod errors;
mod validation;

use essence::EssenceResponse;
pub use hc_crud::{entry_model, Entity, GetEntityInput, UpdateEntityInput};
use hdi::prelude::*;
use serde::de::{Deserializer, Error};

pub use appstore_types::{
    AppEntry, CommonFields, DeprecationNotice, EntityId, LocationTriplet, PublisherEntry,
    WebAddress, WebHappConfig,
};

pub use errors::{AppError, ErrorKinds, UserError};
pub type AppResult<T> = Result<T, ErrorKinds>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub composition: String,
}

pub type Response<T> = EssenceResponse<T, Metadata, ()>;
pub type EntityResponse<T> = Response<Entity<T>>;

pub fn composition<T>(payload: T, composition: &str) -> Response<T> {
    Response::success(
        payload,
        Some(Metadata {
            composition: String::from(composition),
        }),
    )
}

#[hdk_entry_types]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_type]
    Publisher(PublisherEntry),
    #[entry_type]
    App(AppEntry),
}

entry_model!(EntryTypes::Publisher(PublisherEntry));
entry_model!(EntryTypes::App(AppEntry));

#[hdk_link_types]
pub enum LinkTypes {
    Agent,

    Publisher,
    App,

    Anchor,
}

impl<'de> Deserialize<'de> for LinkTypes {
    fn deserialize<D>(deserializer: D) -> Result<LinkTypes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let name: &str = Deserialize::deserialize(deserializer)?;
        match name {
            "Agent" => Ok(LinkTypes::Agent),

            "Publisher" => Ok(LinkTypes::Publisher),
            "App" => Ok(LinkTypes::App),

            "Anchor" => Ok(LinkTypes::Anchor),

            value => Err(D::Error::custom(format!(
                "No LinkTypes value matching '{}'",
                value
            ))),
        }
    }
}

#[macro_export]
macro_rules! catch {
    // could change to "trap", "snare", or "capture"
    ( $r:expr ) => {
        match $r {
            Ok(x) => x,
            Err(e) => {
                let error = match e {
                    appstore::ErrorKinds::AppError(e) => (&e).into(),
                    appstore::ErrorKinds::UserError(e) => (&e).into(),
                    appstore::ErrorKinds::HDKError(e) => (&e).into(),
                    appstore::ErrorKinds::DnaUtilsError(e) => (&e).into(),
                    appstore::ErrorKinds::FailureResponseError(e) => (&e).into(),
                };
                return Ok(appstore::Response::failure(error, None));
            }
        }
    };
    ( $r:expr, $e:expr ) => {
        match $r {
            Ok(x) => x,
            Err(e) => return Ok(appstore::Response::failure((&$e).into(), None)),
        }
    };
}
