use std::sync::PoisonError;

use wasm_bindgen::JsValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error: {0}")]
    Str(String),
    #[error("Error: {0:?}")]
    JsValue(JsValue),
    #[error("Poison Error: {0}")]
    PoisonError(String),
}

impl From<String> for Error{
    fn from(v:String)->Self{
        Self::Str(v)
    }
}
impl From<&str> for Error{
    fn from(v:&str)->Self{
        Self::Str(v.to_string())
    }
}

impl From<JsValue> for Error{
    fn from(v:JsValue)->Self{
        Self::JsValue(v)
    }
}

impl<T> From<PoisonError<T>> for Error 
where T : std::fmt::Debug
{
    fn from(err: PoisonError<T>) -> Error {
        Error::PoisonError(format!("{:?}", err))
    }
}


impl From<Error> for JsValue{
    fn from(err: Error) -> JsValue {
        match err {
            Error::Str(s) | Error::PoisonError(s)=> JsValue::from(s),
            Error::JsValue(v)=>v
        }
    }
}