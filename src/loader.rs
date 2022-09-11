use wasm_bindgen::prelude::*;
use workflow_dom::inject::*;
use workflow_dom::result::Result;


pub fn load_scripts_impl(load : Closure::<dyn FnMut(web_sys::CustomEvent)->Result<()>>) -> Result<()> {

    // let js_script_content = r#" 
    //     alert("hello world");
    // "#.as_bytes();

    let xterm_js = include_bytes!("../extern/resources/xterm.js");
    inject_blob_with_callback("xterm.js", Content::Script(xterm_js), Some(load))?;
    let xterm_addon_fit_js = include_bytes!("../extern/resources/xterm-addon-fit.js");
    inject_blob("xterm-addon-fit.js",Content::Script(xterm_addon_fit_js))?;
    let xterm_addon_web_links_js = include_bytes!("../extern/resources/xterm-addon-web-links.js");
    inject_blob("xterm-addon-web-links.js",Content::Script(xterm_addon_web_links_js))?;
    let xterm_css = include_bytes!("../extern/resources/xterm.css");
    inject_blob("xterm.css", Content::Style(xterm_css))?;
    inject_css("
        .terminal{
            width:100%;
            border:2px solid #DDD;
            min-height:90vh;
        }
    ")?;
    Ok(())
}
