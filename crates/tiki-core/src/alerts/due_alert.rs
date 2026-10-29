use serde::Serialize;

use super::{AlertDelivery, TikiAlert};

/// An alert due now with how it reaches the user, as the shell hands it to the island.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DueAlert {
    #[serde(flatten)]
    pub alert: TikiAlert,
    pub delivery: AlertDelivery,
}
