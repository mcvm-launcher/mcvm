#![warn(missing_docs)]
#![deny(unsafe_code)]

//! This library can install different game modifications, such as Fabric and Paper,
//! for use with the mcvm_core library. It is packaged separately so that users who
//! do not want this functionality and only act on the vanilla game don't have to include
//! them

/// Installation of the Fabric and Quilt modloaders
pub mod fabric_quilt;
