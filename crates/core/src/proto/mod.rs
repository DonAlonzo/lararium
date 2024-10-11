mod adoption;
mod controller;

tonic::include_proto!("lararium");

pub const DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor");
