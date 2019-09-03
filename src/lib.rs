extern crate nix;
extern crate libc;

use std::ffi::CString;
use std::path::Path;
//use libc::pid_t;
use nix::unistd::*;
use nix::sys::signal;
use nix::sys::ptrace;
use nix::sys::wait::*;
use nix::Error;
use nix::errno::Errno;

pub mod inferior;
pub mod debug;
use inferior::*;
