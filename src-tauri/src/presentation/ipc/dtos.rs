use std::borrow::Borrow;

use serde::Serialize;

use crate::application::error::Error;

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
