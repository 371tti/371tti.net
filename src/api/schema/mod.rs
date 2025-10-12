pub mod auth;
pub mod search;

use std::ops::Range;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::user_manager::auth_manager::{AccountID, AccountSession};



// --- Search API schema ---
