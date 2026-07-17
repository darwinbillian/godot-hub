use std::borrow::Borrow;

use anyhow::Error;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ErrorDto {
    message: String,
}

impl<E> From<E> for ErrorDto
where
    E: Borrow<Error>,
{
    fn from(value: E) -> Self {
        let value = value.borrow();
        Self {
            message: value.to_string(),
        }
    }
}
