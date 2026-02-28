/*
MIT License

Copyright (c) 2026 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

pub mod hints;
pub mod protocol;

use std::{array::TryFromSliceError, string::FromUtf8Error};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NamingError {
    #[error("Name {name} has size {len} above max limit.", len = .name.len())]
    InvalidNameSize { name: String },
    #[error("Label {label} size {len} above max limit.", len = .label.len())]
    InvalidLabelSize { label: String },
}

#[derive(Error, Debug)]
pub enum TextError {
    #[error("Text has size {len} above max limit.", len = .text.len())]
    InvalidNameSize { text: String },
}

#[derive(Error, Debug)]
#[error("Error: {0}")]
pub struct UnexpectedValueError(String);

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Name has size {actual} above max limit {expected_max}.")]
    InvalidStringSize { expected_max: usize, actual: usize },

    #[error("The size of the slice is not compatible with the ongoing processing")]
    InvalidSliceSize,

    #[error(transparent)]
    InvalidSliceConversion(#[from] TryFromSliceError),

    #[error(transparent)]
    CharacterConversion(#[from] FromUtf8Error),

    #[error(transparent)]
    UnexpectedValue(#[from] UnexpectedValueError),
}

#[derive(Error, Debug)]
pub enum QueryMessageError {
    #[error("Questions count is zero.")]
    NoQuestions,
    #[error("The message does not have the query flag")]
    NotQuery,
}

#[derive(Error, Debug)]
pub enum ResponseMessageError {
    #[error("Questions count is zero.")]
    NoQuestions,
    #[error("The message does not have the query flag")]
    NotResponse,
}
