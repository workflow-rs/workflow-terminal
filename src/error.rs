use wasm_bindgen::JsValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Error: {0}")]
    Str(String),
    #[error("Error: {0:?}")]
    JsValue(JsValue)
}

impl From<String> for Error{
    fn from(v:String)->Self{
        Self::Str(v)
    }
}

impl From<JsValue> for Error{
    fn from(v:JsValue)->Self{
        Self::JsValue(v)
    }
}