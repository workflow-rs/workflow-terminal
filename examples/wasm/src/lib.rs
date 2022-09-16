// use workflow_terminal::{CliHandler, Result, Terminal};
// use workflow_terminal::cli::Terminal as TerminalTrait;
// use workflow_terminal::{on_terminal_ready, get_terminal, load_scripts};
// use workflow_log::*;
// use wasm_bindgen::prelude::*;
// use std::sync::Arc;
// use async_trait::async_trait;

// struct TestHandler{
//     term:Arc<Terminal>
// }

// impl TestHandler{
//     fn new(term:Arc<Terminal>)->Result<Arc<Self>>{
//         let handler = Arc::new(Self{term});
//         handler.term.register_handler(handler.clone())?;
//         Ok(handler)
//     }
// }

// //static PP: Option<js_sys::Promise> = None;
// async fn digest(cmd:String){
//     log_trace!("digest:cmd:{}", cmd);
//     /*let a = XYZ::new();
//     let v = a.fetch();
//     let promise = js_sys::Promise::from(PP);
//     unsafe {
//         PP = Some(promise);
//     }
//     //let promise = _test_abc_();
    
//     let result = wasm_bindgen_futures::JsFuture::from(promise).await;
//     */
// }

// //#[wasm_bindgen(inline_js = "export function _test_abc_() { return new Promise(resolve=>{setTimeout(()=>resolve(true), 3)})  }")]
// /*#[wasm_bindgen]
// extern "C" {
//     //#[wasm_bindgen(js_namespace=window)]
//     //async fn _test_abc_();

//     #[wasm_bindgen(js_namespace=window, js_name="_XYZ")]
//     type XYZ;

//     #[wasm_bindgen(constructor, js_class = "_XYZ")]
//     fn new() -> XYZ;

//     #[wasm_bindgen(method)]
//     fn fetch(this: &XYZ)->JsValue;
// }

// unsafe impl Send for XYZ{}
// unsafe impl Sync for XYZ{}
// */


// #[async_trait]
// impl CliHandler for TestHandler{
//     async fn digest(&self, cmd:String)->Result<()> {
//         let term = self.term.clone();
//         log_trace!("cmd:{}", cmd);
//         digest(cmd.clone()).await;
//         //let five_msec = time::Duration::from_millis(5000);
//         //sleep(five_msec).await;
//         if cmd.eq("hello"){
//             term.write_str("Hi from wasm example")?;
//         }else{
//             term.write_str("Please use 'hello' command")?;
//         }
//         Ok(())
//     }

//     async fn complete(&self, substring : String) -> Result<Vec<String>>{
//         if substring.starts_with("h"){
//             return Ok(Vec::from(["Hello".to_string(), "h".to_string()]));
//         }
//         Ok(Vec::new())
//     }
// }

// #[wasm_bindgen(start)]
// pub fn boot()->Result<()>{
//     on_terminal_ready(Box::new(||->Result<()>{
//         let term = get_terminal()?;
//         let h = TestHandler::new(term)?;
//         h.term.write_str("Example of wasm Terminal:")?;
//         h.term.prompt()?;
//         Ok(())
//     }));

//     load_scripts()?;
//     Ok(())
// }




// /*
// static mut CLI : Option<Arc<Cli>> = None;
// #[wasm_bindgen(js_name="testCli")]
// pub fn test_cli()->Result<()>{
//     let term = get_terminal()?;
//     let prompt = Arc::new(Mutex::new("$ ".to_string()));
//     let cli = Arc::new(Cli::new(term, prompt)?);
//     cli.prompt()?;
//     cli.write("TESTING CLI...")?;
//     cli.set_handler(Arc::new(CliHandler::new(cli.clone())))?;

//     unsafe { CLI = Some(cli); }
//     Ok(())
// }
// */


use wasm_bindgen::prelude::*;
use workflow_terminal::Result;
use cli::example_terminal;

#[wasm_bindgen(start)]
pub async fn load_terminal() -> Result<()>{
    example_terminal().await?;
    Ok(())
}
