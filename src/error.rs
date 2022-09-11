use wasm_bindgen::JsValue;

pub enum Error{
    Str(String),
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