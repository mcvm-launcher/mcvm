/// Installing and launching proxies on profiles
pub mod proxy;
use mcvm_shared::id::InstanceRef;

use super::instance::Instance;

/// A hashmap of InstanceIDs to Instances
pub type InstanceRegistry = std::collections::HashMap<InstanceRef, Instance>;
