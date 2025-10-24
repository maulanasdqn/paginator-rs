use paginator_utils;
pub use paginator_utils::*;

mod error;
mod trait_impl;
mod builder;

pub use error::{PaginatorError, PaginatorResult};
pub use trait_impl::PaginatorTrait;
pub use builder::PaginatorBuilder;
