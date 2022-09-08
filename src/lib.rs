// use js_sys::{ArrayBuffer,Uint8Array};
use js_sys::*;//{Array,Uint8Array};
use wasm_bindgen::prelude::*;
use web_sys::{Url,Blob};
/*

const source = "alert('test')";
const el = document.createElement("script");
el.src = URL.createObjectURL(new Blob([source], { type: 'text/javascript' }));
document.head.appendChild(el);

*/


// #[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    // use super::*;
    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = console)]
        pub fn log(s: &str);
        #[wasm_bindgen(js_namespace = console)]
        pub fn warn(s: &str);
        #[wasm_bindgen(js_namespace = console)]
        pub fn error(s: &str);
    }
}

#[macro_export]
macro_rules! log_trace {
    ($($t:tt)*) => (
        crate::wasm::log(format_args!($($t)*).to_string().as_str())
    )
}



pub enum Content<'content> {
    Script(&'content [u8]),
    Style(&'content [u8]),
}


pub fn load_scripts_impl() -> Result<(),JsValue> {
    let xterm_js = include_bytes!("../extern/resources/xterm.js");
    inject_blob("xterm.js", Content::Script(xterm_js))?;
    let xterm_addon_fit_js = include_bytes!("../extern/resources/xterm-addon-fit.js");
    inject_blob("xterm-addon-fit.js",Content::Script(xterm_addon_fit_js))?;
    let xterm_addon_web_links_js = include_bytes!("../extern/resources/xterm-addon-web-links.js");
    inject_blob("xterm-addon-web-links.js",Content::Script(xterm_addon_web_links_js))?;
    let xterm_css = include_bytes!("../extern/resources/xterm.css");
    inject_blob("xterm.css",Content::Style(xterm_css))?;

    Ok(())
}


// pub fn inject_blob(js_script_content : &[u8]) -> Result<(),JsValue> {
pub fn inject_blob(name : &str, content : Content) -> Result<(),JsValue> {

    log_trace!("loading {}",name);
    // let js_script_content = r#"
    //     alert("hello world");
    // "#.as_bytes();

    let document = web_sys::window().unwrap().document().unwrap();
    let html_root = document.get_elements_by_tag_name("html").item(0).unwrap();

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
        
            let script = document.create_element("script")?;
            script.set_attribute("type","text/javascript")?;
            script.set_attribute("src",&url)?;
            html_root.append_child(&script)?;
        },
        Content::Style(content) => {
            let args = Array::new_with_length(1);
            args.set(0, unsafe { Uint8Array::view(content).into() });
            let blob = Blob::new_with_u8_array_sequence(&args)?;
            let url = Url::create_object_url_with_blob(&blob)?;
        
            let style = document.create_element("style")?;
            style.set_attribute("type","text/stylesheet")?;
            style.set_attribute("src",&url)?;
            html_root.append_child(&style)?;
        },
    }

    Ok(())
}


#[wasm_bindgen(start)]
pub fn load_scripts() {
    load_scripts_impl().unwrap();
}

// #[cfg(test)]
// mod tests {

//     use std::include_bytes;

//     #[test]
//     fn it_works() {

//         // assert_eq!(bytes, b"adi\xc3\xb3s\n");
//         print!("{}", String::from_utf8_lossy(bytes));


//         // let result = 2 + 2;
//         // assert_eq!(result, 4);
//     }
// }
