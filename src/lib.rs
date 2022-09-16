// use js_sys::*;
// use wasm_bindgen::JsCast;
// use cfg_if::cfg_if;
// use web_sys::{Url,Blob};

pub mod error;
pub mod result;
pub mod loader;
pub mod keys;
pub mod cursor;
pub mod terminal;

//use js_sys::Promise;
pub use result::Result;
pub use terminal::Cli;
pub use terminal::Terminal;
pub use terminal::Options;
pub use terminal::parse;

// use std::future::Future;
//use std::time;
//use async_std::task::sleep;

// use std::sync::Arc;
// use std::sync::Mutex;





// pub mod listener;
// pub mod utils;
// use workflow_dom::*;

//use error::Error;
// pub use listener::Listener;
// pub use utils::{body, document};


// pub type Result<T> = std::result::Result<T, JsValue>;

// #[cfg(target_arch = "wasm32")]
// pub mod wasm {
//     use wasm_bindgen::prelude::*;
//     // use super::*;
//     #[wasm_bindgen]
//     extern "C" {
//         #[wasm_bindgen(js_namespace = console)]
//         pub fn log(s: &str);
//         #[wasm_bindgen(js_namespace = console)]
//         pub fn warn(s: &str);
//         #[wasm_bindgen(js_namespace = console)]
//         pub fn error(s: &str);
//     }
// }

// #[macro_export]
// macro_rules! log_trace {
//     ($($t:tt)*) => (
//         crate::wasm::log(format_args!($($t)*).to_string().as_str())
//     )
// }

/* 
pub enum Content<'content> {
    Script(&'content [u8]),
    Style(&'content [u8])
}


pub fn inject_css(css : &str) -> Result<()> {
    let doc = document();
    let head = doc.get_elements_by_tag_name("head").item(0).ok_or("")?;
    let style_el = doc.create_element("style")?;
    style_el.set_inner_html(css);
    head.append_child(&style_el)?;
    Ok(())
}

pub fn inject_blob(name : &str, content : Content) -> Result<()> {

    log_trace!("loading {}",name);

    let doc = document();
    let html_root = doc.get_elements_by_tag_name("body").item(0).unwrap();

    let mime = js_sys::Object::new();
    js_sys::Reflect::set(&mime, &"type".into(), &JsValue::from_str("text/javascript"))?;
    
    match content {
        Content::Script(content) => {

            let string = String::from_utf8_lossy(content);
            let regex = regex::Regex::new(r"//# sourceMappingURL.*$").unwrap();
            let content = regex.replace(&string, "");

            let args = Array::new_with_length(1);
            args.set(0, unsafe { Uint8Array::view(content.as_bytes()).into() });
            let blob = Blob::new_with_u8_array_sequence(&args)?;
            let url = Url::create_object_url_with_blob(&blob)?;
        
            let script = doc.create_element("script")?;
            script.set_attribute("type","text/javascript")?;
            script.set_attribute("src", &url)?;
            if name.eq("xterm.js"){
                let listener = Closure::<dyn FnMut(web_sys::CustomEvent)->Result<()>>::new(move|_: web_sys::CustomEvent|->Result<()>{
                    log_trace!("init_terminal...");
                    //inject_init_terminal()?;
                    init_terminal()?;
                    Ok(())
                });
                script.add_event_listener_with_callback("load", listener.as_ref().unchecked_ref())?;
                listener.forget();
            }
            html_root.append_child(&script)?;
        },
        Content::Style(content) => {
            let args = Array::new_with_length(1);
            args.set(0, unsafe { Uint8Array::view(content).into() });
            let blob = Blob::new_with_u8_array_sequence(&args)?;
            let url = Url::create_object_url_with_blob(&blob)?;
        
            let style = doc.create_element("link")?;
            style.set_attribute("type","text/css")?;
            style.set_attribute("rel","stylesheet")?;
            style.set_attribute("href",&url)?;
            html_root.append_child(&style)?;
        },
    }

    Ok(())
}
*/


/*
pub fn inject_init_terminal()->Result<()>{
    let init_script = r#"
        terminal_wasm.init_terminal();
    "#.as_bytes();
    inject_blob("init.js", Content::Script(init_script))?;
    Ok(())
}
*/

// #[cfg(test)]
// mod test{
//     #[test]
//     pub fn cli(){
//         println!("cli test1");
//     }
// }