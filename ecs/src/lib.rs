#![feature(core_intrinsics)]

extern crate slab;
extern crate atom;
extern crate fnv;
extern crate map;
extern crate listener;
extern crate pointer;
#[macro_use]
extern crate any;

extern crate im;

pub mod world;
pub mod system;
pub mod entity;
pub mod component;

pub mod idtree;
pub mod dispatch;
pub mod dirty;
pub trait Share: Send + Sync + 'static {

}