// use wasm_bindgen::prelude::*;
use workflow_dom::inject::*;
// use workflow_dom::result::Result;

use web_sys::Element;
use workflow_dom::utils::*;
use workflow_log::*;
use crate::Result;
// use crate::cli::{Intake, Terminal as TerminalTrait, CliHandler, DefaultHandler};
use crate::keys::Key;
use crate::terminal::Terminal;
use crate::terminal::Options;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::{Mutex, Arc};
use workflow_wasm::listener::Listener;
use workflow_wasm::utils::*;
use workflow_core::channel::{oneshot,unbounded,Sender,Receiver};

/// #[wasm_bindgen(module = "/defined-in-js.js")]
#[wasm_bindgen()]
extern "C" {

    #[wasm_bindgen(js_namespace=window, js_name="Terminal")]
    type XtermImpl;

    #[wasm_bindgen(constructor, js_class = "Terminal")]
    fn new(opt: js_sys::Object) -> XtermImpl;

    #[wasm_bindgen(method, getter)]
    fn number(this: &XtermImpl) -> u32;

    #[wasm_bindgen(method)]
    fn open(this: &XtermImpl, el: &Element);

    #[wasm_bindgen(method, js_name="onKey")]
    fn on_key(this: &XtermImpl, f: &js_sys::Function);

    #[wasm_bindgen(method, js_name="write")]
    fn _write(this: &XtermImpl, text:String);
}

impl Debug for XtermImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "TODO::")?;
        Ok(())
    }
}

impl XtermImpl {
    fn write<T:Into<String>>(&self, text:T){
        self._write(text.into());
    }
}

enum Ctl {
    SinkEvent(SinkEvent),
    Close
}

#[derive(Debug)]
pub struct SinkEvent{
    key:String,
    term_key:String,
    ctrl_key:bool,
    alt_key:bool,
    meta_key:bool,
}

impl SinkEvent{
    fn new(
        key:String,
        term_key:String,
        ctrl_key:bool,
        alt_key:bool,
        meta_key:bool,
    )->Self{
        Self{key, term_key, ctrl_key, alt_key, meta_key}
    }
}



use workflow_dom::utils::body;
use wasm_bindgen::prelude::*;
// use workflow_log::*;
// use wasm_bindgen_futures::spawn_local;

// pub fn spawn<F>(future: F) where F: Future<Output = ()> + 'static{
//     spawn_local(future)
// }

// pub fn get_terminal() -> Result<Arc<Interface>> {
//     let term = unsafe { (&TERMINAL).as_ref().unwrap().clone() };
//     Ok(term.clone())
// }


// static mut TERMINAL : Option<Arc<Terminal>> = None;
// static mut INIT_FN : Option<Box<dyn Fn()->Result<()>>> = None;

// pub fn init_terminal()->Result<()>{
//     let body_el = body()?;
//     let terminal = Terminal::new(&body_el, Options{
//         prompt:"$ ".to_string()
//     })?;
//     unsafe { TERMINAL = Some(terminal); }

//     if let Some(init_fn) = unsafe { (&INIT_FN).as_ref() }{
//         init_fn()?;
//     }

//     Ok(())
// }

// pub fn on_terminal_ready(f: Box<dyn Fn()->Result<()>>){
//     unsafe { INIT_FN = Some(f); }
// }




// pub struct Options{
//     pub prompt:String
// }

#[derive(Clone)]
pub struct Sink {
    receiver : Receiver<Ctl>,
    sender : Sender<Ctl>,
}

impl Sink {
    pub fn new() -> Sink {
        let (sender, receiver) = unbounded();
        Sink {
            receiver,
            sender,
        }
    }
}

unsafe impl Send for Xterm{}
unsafe impl Sync for Xterm{}

pub struct Xterm {
    pub element: Element,
    xterm:Arc<Mutex<Option<XtermImpl>>>,
    terminal: Arc<Mutex<Option<Arc<Terminal>>>>,
    listener: Arc<Mutex<Option<Listener<JsValue>>>>,
    sink : Arc<Sink>,

    // intake: Arc<Intake>,
    // handler: Arc<Mutex<Arc<dyn CliHandler>>>,
    //cli: Option<Cli>
}

impl Xterm{

    pub fn try_new() -> Result<Self> {
        Self::new_with_options(Options::default())
    }

    pub fn new_with_options(options: Options) -> Result<Self> {
        let body_el = body().expect("Unable to get 'body' element");
        Self::new_with_element(&body_el, options)
    }

    pub fn new_with_element(parent:&Element, _options:Options)->Result<Self> {
        let element = document().create_element("div")?;
        element.set_attribute("class", "terminal")?;
        parent.append_child(&element)?;
        let terminal = Xterm{
            element,
            listener: Arc::new(Mutex::new(None)),
            // xterm: Self::create_term()?,
            xterm: Arc::new(Mutex::new(None)),
            terminal: Arc::new(Mutex::new(None)),
            sink : Arc::new(Sink::new())

            // intake: Arc::new(Intake::new(Arc::new(Mutex::new(opt.prompt)))?),
            // handler: Arc::new(Mutex::new(Arc::new(DefaultHandler::new()))),
            //cli: None
        };
        // let term = terminal.init()?;
        Ok(terminal)
    }

    fn init_xterm()->Result<XtermImpl>{
        let theme = js_sys::Object::new();
        let theme_opts = Vec::from([
            ("background", JsValue::from("rgba(255,255,255,1)")),
			("foreground", JsValue::from("#000")),
            // ("background", JsValue::from("rgba(0,0,0,1)")),
			// ("foreground", JsValue::from("#FFF")),
			("cursor", JsValue::from("#000"))
        ]);
        for (k, v) in theme_opts{
            js_sys::Reflect::set(&theme, &k.into(), &v)?;
        }

        let options = js_sys::Object::new();
        let opts = Vec::from([
            ("allowTransparency", JsValue::from(true)),
            ("fontFamily", JsValue::from("Consolas, Ubuntu Mono, courier-new, courier, monospace")),
            ("fontSize", JsValue::from(20)),
            ("cursorBlink", JsValue::from(true)),
            ("theme", JsValue::from(theme))
        ]);
        for (k, v) in opts{
            js_sys::Reflect::set(&options, &k.into(), &v)?;
        }
        
        let term = XtermImpl::new(options);
        log_trace!("term: {:?}", term);


        Ok(term)
    }

    pub async fn init(self : &Arc<Self>, terminal : &Arc<Terminal>)->Result<()>{
        log_trace!("Terminal.init()....");

        let receiver = load_scripts()?;
        receiver.recv().await?;

        let xterm = Self::init_xterm()?;

        xterm.open(&self.element);

        // let self_arc = Arc::new(self);
        let this = self.clone();
        let listener = Listener::new(move |e|->std::result::Result<(), JsValue>{
            let term_key = try_get_string(&e, "key")?;
            //log_trace!("on_key: {:?}, key:{}", e, key);
            let dom_event = try_get_js_value(&e, "domEvent")?;
            let ctrl_key = try_get_bool_from_prop(&dom_event, "ctrlKey").unwrap_or(false);
            let alt_key = try_get_bool_from_prop(&dom_event, "altKey").unwrap_or(false);
            let meta_key = try_get_bool_from_prop(&dom_event, "metaKey").unwrap_or(false);
            
            let key_code = try_get_u64_from_prop(&dom_event, "keyCode")?;
            let key = try_get_string(&dom_event, "key")?;
            log_trace!("key_code: {}, key:{}, ctl_key:{}", key_code, key, ctrl_key);
            //let locked = this.lock().expect("msg");
            this.sink.sender.try_send(

                Ctl::SinkEvent(SinkEvent::new(key, term_key, ctrl_key, alt_key, meta_key))

            ).unwrap();

            // this.sink(SinkEvent::new(key, term_key, ctrl_key, alt_key, meta_key), e)?;
            
            Ok(())
        });
        //let locked = self_arc.lock().expect("Unable to lock terminal");
        xterm.on_key(listener.into_js());
        // let self_arc_clone = self_arc.clone();
        
        //let mut locked_listener =  
        *self.listener.lock().unwrap() = Some(listener);
        *self.xterm.lock().unwrap() = Some(xterm);
        *self.terminal.lock().unwrap() = Some(terminal.clone());
        
        Ok(())
    }

    pub async fn run(self: &Arc<Self>) -> Result<()> {

        loop {
            let event = self.sink.receiver.recv().await?;
            match event {
                Ctl::SinkEvent(event) => {
                    self.sink(event).await?;
                },
                Ctl::Close => {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn exit(&self) {
        self.sink.sender.try_send(Ctl::Close).expect("Unable to send exit Ctl");
    }

    // async fn sink(&self, e:SinkEvent, _e:JsValue)->Result<()>{
    async fn sink(&self, e:SinkEvent)->Result<()>{

        let key = 

        match e.key.as_str(){
            "Backspace" => Key::Backspace,
            "ArrowUp"=> Key::ArrowUp,
            "ArrowDown"=> Key::ArrowDown,
            "ArrowLeft"=> Key::ArrowLeft,
            "ArrowRight"=>Key::ArrowRight,
            "Escape"=>Key::Esc,
            "Delete"=>Key::Delete,
            "Tab"=>{
                //TODO
                return Ok(());
            }
            // "Inject"=>{
            //     inject(term_key);
            // }
            "Enter" => Key::Enter,
            _=>{
                let printable = !e.meta_key; // ! (e.ctrl_key || e.alt_key || e.meta_key);
                if !printable{
                    return Ok(());
                }
                //log_trace!("Char: {}", e.key);
                if let Some(c) = e.key.chars().next(){
                    //log_trace!("Char2: {}, {}", e.key, c);
                    //log_trace!("e:{:?}", e);
                    if e.ctrl_key{
                        Key::Ctrl(c)
                    }else{
                        if e.alt_key{
                            Key::Alt(c)
                        }else{
                            Key::Char(c)
                        }
                    }
                }else{
                    return Ok(());
                }
            }
        };

        
        let _res = self.terminal
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .ingest(key, e.term_key).await?;
        



        // for text in res.texts{
        //     self.xterm.write(text);
        // }
        // if let Some(cmd) = res.cmd{
        //     self.digest(cmd)?;
        // }

        /*
        if let Some(cli) = self.cli.as_ref(){
            cli.intake(key, e.term_key)?;
        }
        */


        Ok(())
    }

    // pub fn write_str<S>(&self, text:S)->Result<()> where S:Into<String>{
    //     self.xterm.write(text.into());
    //     Ok(())
    // }

    // pub fn prompt(&self)->Result<()>{
    //     self.xterm.write(self.intake.prompt()?);
    //     Ok(())
    // }

    // pub fn inner(&self) -> LockResult<MutexGuard<'_, Inner>> {
    //     self.inner.lock()
    // }


    pub fn write<S>(&self, s:S) where S:Into<String>{
        self.xterm
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .write(s.into());
    }

    
    // pub fn writeln<S>(&self, s:S) where S:Into<String>{
    //     self.xterm.write(format!("{}\n\x1B[2K\r", s.into()));
    //     // stdout.flush().unwrap();

    // }


    // fn write_vec(&self, mut str_list:Vec<String>) ->Result<()> {
    //     let data = self.intake.inner()?;
		
    //     str_list.push("\r\n".to_string());
        
	// 	if self.intake.is_running() {
	// 		self.xterm.write(str_list.join(""));
	// 	}else {
	// 		self.xterm.write(format!("\x1B[2K\r{}", str_list.join("")));
	// 		let prompt = format!("{}{}", self.intake.prompt_str(), data.buffer.join(""));
	// 		self.xterm.write(prompt);
	// 		let l = data.buffer.len() - data.cursor;
	// 		for _ in 0..l{
	// 			self.xterm.write("\x08".to_string());
    //         }
	// 	}

    //     Ok(())
	// }

    // fn _write<S>(&self, s : S)->Result<()> where S : Into<String> {
    //     self.xterm.write(s.into());
    //     Ok(())
    // }

	/*
    fn write<S>(&self, s : S)-> Result<()>  where S : Into<String> {
        let s:String = s.into();
		self.write_vec(Vec::from([s]))?;
        Ok(())
	}
    */

  
}


// impl TerminalTrait for Xterm{
//     fn write(&self, s: String) -> Result<()>{
//         //self.xterm.write(s);
//         self.write_vec(Vec::from([s]))?;
//         Ok(())
//     }

//     fn start(&self)-> Result<()> {
//         //self._start()?;
//         Ok(())
//     }

//     fn digest(&self, cmd: String) -> Result<()>{
//         let hander = self.handler.clone();
//         let intake = self.intake.clone();
//         let term = self.xterm.clone();
//         crate::spawn(async move{
//             let locked = hander.lock().expect("Unable to lock terminal.handler for digest");
//             let _r = locked.digest(cmd).await;
//             match intake.after_digest(){
//                 Ok(text)=>{
//                     let _r = utils::apply_with_args1(&term, "write", JsValue::from(text));
//                 }
//                 Err(_e)=>{
//                     //
//                 }
//             }
//         });
//         Ok(())
//     }

//     fn register_handler(&self, hander: Arc<dyn CliHandler>)-> Result<()> {
//         let mut locked = self.handler.lock().expect("Unable to lock terminal.handler");
//         *locked = hander;
//         Ok(())
//     }
// }


static mut XTERM_LOADED: bool = false;

pub fn load_scripts() ->Result<Receiver<()>> {
    let (sender, receiver) = oneshot();

    if unsafe { XTERM_LOADED } {
        sender.try_send(()).expect("Unable to send xterm loaded");
        return Ok(receiver);
    }

    load_scripts_impl(Closure::<dyn FnMut(web_sys::CustomEvent)->std::result::Result<(), JsValue>>::new(move|_|->std::result::Result<(), JsValue>{
        log_trace!("init_terminal...");
        unsafe { XTERM_LOADED = true };
        sender.try_send(()).expect("Unable to send xterm loaded");
        Ok(())
    }))?;
    Ok(receiver)
}


pub fn load_scripts_impl(load : Closure::<dyn FnMut(web_sys::CustomEvent)->std::result::Result<(),JsValue>>) -> Result<()> {

    // let js_script_content = r#" 
    //     alert("hello world");
    // "#.as_bytes();

    let xterm_js = include_bytes!("../../extern/resources/xterm.js");
    inject_blob_with_callback("xterm.js", Content::Script(xterm_js), Some(load))?;
    let xterm_addon_fit_js = include_bytes!("../../extern/resources/xterm-addon-fit.js");
    inject_blob("xterm-addon-fit.js",Content::Script(xterm_addon_fit_js))?;
    let xterm_addon_web_links_js = include_bytes!("../../extern/resources/xterm-addon-web-links.js");
    inject_blob("xterm-addon-web-links.js",Content::Script(xterm_addon_web_links_js))?;
    let xterm_css = include_bytes!("../../extern/resources/xterm.css");
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
