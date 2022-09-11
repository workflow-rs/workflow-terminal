use wasm_bindgen::prelude::*;
use web_sys::{Element, Window, Document};
use js_sys::{Uint8Array, Array};

pub fn window() -> Window{
    web_sys::window().unwrap()
}

pub fn document() -> Document{
    web_sys::window().unwrap().document().unwrap()
}

pub fn body()->std::result::Result<Element, String>{
    let b = document().query_selector("body").unwrap().ok_or("Unable to get body element".to_string())?;
    Ok(b)
}

pub fn js_value(this_jsv:&JsValue, prop:&str)->Result<JsValue,JsValue> {
    let v = js_sys::Reflect::get(&this_jsv, &JsValue::from(prop))?;
    Ok(v)
}

pub fn apply_with_args0(this_jsv: &JsValue, fn_name: &str) -> Result<JsValue,JsValue> {
    let fn_jsv = js_sys::Reflect::get(&this_jsv,&JsValue::from(fn_name))?;
    let args = Array::new();
    let ret_jsv = js_sys::Reflect::apply(&fn_jsv.into(),&this_jsv,&args.into())?;
    Ok(ret_jsv)
}

pub fn apply_with_args1(this_jsv: &JsValue, fn_name: &str, arg_jsv : JsValue) -> Result<JsValue,JsValue> {
    let fn_jsv = js_sys::Reflect::get(&this_jsv,&JsValue::from(fn_name))?;
    let args = Array::new_with_length(1);
    args.set(0, arg_jsv);
    let ret_jsv = js_sys::Reflect::apply(&fn_jsv.into(),&this_jsv,&args.into())?;
    Ok(ret_jsv)
}
pub fn apply_with_args2(this_jsv: &JsValue, fn_name: &str, arg_jsv : JsValue, arg2_jsv : JsValue) -> Result<JsValue,JsValue> {
    let fn_jsv = js_sys::Reflect::get(&this_jsv,&JsValue::from(fn_name))?;
    let args = Array::new_with_length(2);
    args.set(0, arg_jsv);
    args.set(1, arg2_jsv);
    let ret_jsv = js_sys::Reflect::apply(&fn_jsv.into(),&this_jsv,&args.into())?;
    Ok(ret_jsv)
}

/// Returns successfully parsed value or 0
pub fn try_get_u64_from_prop(jsv : &JsValue, prop : &str) -> Result<u64,JsValue> {
    Ok(js_sys::Reflect::get(&jsv,&JsValue::from(prop))?.as_f64()
        .ok_or(JsValue::from(format!("try_get_u64(): error parsing {}",prop)))?
        as u64
    )
}
pub fn try_get_u8_from_prop(jsv : &JsValue, prop : &str) -> Result<u8,JsValue> {
    Ok(js_sys::Reflect::get(&jsv,&JsValue::from(prop))?.as_f64()
        .ok_or(JsValue::from(format!("try_get_u8(): error parsing {}",prop)))?
        as u8
    )
}

pub fn try_get_bool_from_prop(jsv : &JsValue, prop : &str) -> Result<bool,JsValue> {
    Ok(js_sys::Reflect::get(&jsv,&JsValue::from(prop))?.as_bool()
        .ok_or(JsValue::from(format!("try_get_bool(): property {} is missing or not a boolean",prop)))?
    )
}

pub fn try_get_vec_from_prop(jsv : &JsValue, prop : &str) -> Result<Vec<u8>,JsValue> {
    let buffer = js_sys::Reflect::get(&jsv,&JsValue::from(prop))?;
    let array = Uint8Array::new(&buffer);
    let data: Vec<u8> = array.to_vec();
    Ok(data)
}

pub fn try_get_vec_from_bn_prop(object_jsv : &JsValue, prop : &str) -> Result<Vec<u8>,JsValue> {

    let bn_jsv = js_sys::Reflect::get(&object_jsv,&JsValue::from(prop))?;
    let bytes = apply_with_args0(&bn_jsv, "toBytes" )?;
    let array = Uint8Array::new(&bytes);
    Ok(array.to_vec())
}

pub fn try_get_vec_from_bn(bn_jsv : &JsValue) -> Result<Vec<u8>, JsValue> {

    let bytes = apply_with_args0(&bn_jsv, "toBytes" )?;
    let array = Uint8Array::new(&bytes);
    Ok(array.to_vec())
}

pub fn try_get_string(jsv : &JsValue, prop : &str) -> Result<String, JsValue> {
    let str = js_sys::Reflect::get(jsv, &JsValue::from(prop))?;
    match str.as_string(){
        Some(str)=>Ok(str),
        None=>{
            return Err(JsValue::from(format!("Unable to find property '{}' on object '{:?}'", prop, jsv)));
        }
    }
}
