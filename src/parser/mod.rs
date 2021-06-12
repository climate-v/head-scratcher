//! Main parsing module
//!
//! # Parser
//! Main parsing module
use crate::error::HeadScratcherError as HSE;
use nom::IResult;

pub mod components;

pub type HSEResult<I, O> = IResult<I, O, HSE<I>>;
