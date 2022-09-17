use workflow_dom::inject::*;
use web_sys::{ Element, EventTarget };
use workflow_dom::utils::*;
use workflow_log::*;
use crate::Result;
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
use workflow_dom::utils::body;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace=["window", "FitAddon"], js_name="FitAddon")]
    type FitAddon;

    #[wasm_bindgen(constructor, js_class = "window.FitAddon.FitAddon", js_name="FitAddon")]
    fn new() -> FitAddon;

    #[wasm_bindgen(method, js_name="proposeDimensions")]
    fn propose_dimensions(this: &FitAddon);

    #[wasm_bindgen(method, js_name="fit")]
    fn fit(this: &FitAddon);
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_namespace=["window","WebLinksAddon"], js_name="WebLinksAddon")]
    type WebLinksAddon;

    #[wasm_bindgen(constructor, js_class = "window.WebLinksAddon.WebLinksAddon", js_name = "WebLinksAddon")]
    fn new(callback : JsValue) -> WebLinksAddon;
}

#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(extends = js_sys::Object)]
    type XtermEvent;

    #[wasm_bindgen(method, getter, js_name="domEvent")]
    fn get_dom_event(this: &XtermEvent) -> web_sys::KeyboardEvent;
    #[wasm_bindgen(method, getter, js_name="key")]
    fn get_key(this: &XtermEvent) -> String;

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

    // #[wasm_bindgen(method, js_name="paste")]
    // fn _paste(this: &XtermImpl, text:String);

    #[wasm_bindgen(method, js_name="loadAddon")]
    fn load_addon(this: &XtermImpl, addon : JsValue);

    #[wasm_bindgen(method, getter, js_name="element")]
    fn get_element(this: &XtermImpl)->Element;
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
    // fn paste<T:Into<String>>(&self, text:T){
    //     self._paste(text.into());
    // }
}

enum Ctl {
    SinkEvent(SinkEvent),
    Paste,
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

///
/// # Xterm 
/// 
/// Wrapper around XtermJS - https://github.com/xtermjs/xterm.js
/// 
/// TODO enhance API to match https://github.com/xtermjs/xterm.js/blob/4.14.1/typings/xterm.d.ts
/// 
/// 
pub struct Xterm {
    pub element: Element,
    xterm:Arc<Mutex<Option<XtermImpl>>>,
    terminal: Arc<Mutex<Option<Arc<Terminal>>>>,
    listener: Arc<Mutex<Option<Listener<XtermEvent>>>>,
    sink : Arc<Sink>,
    resize : Arc<Mutex<Option<(ResizeObserver,Listener<JsValue>)>>>,
    fit : Arc<Mutex<Option<FitAddon>>>,
    web_links : Arc<Mutex<Option<WebLinksAddon>>>,
    clipboard_listerner:  Arc<Mutex<Option<Listener<web_sys::KeyboardEvent>>>>,
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
            xterm: Arc::new(Mutex::new(None)),
            terminal: Arc::new(Mutex::new(None)),
            sink : Arc::new(Sink::new()),
            resize: Arc::new(Mutex::new(None)),
            // addons: Arc::new(Mutex::new(Vec::new())),
            fit : Arc::new(Mutex::new(None)),
            web_links : Arc::new(Mutex::new(None)),
            clipboard_listerner: Arc::new(Mutex::new(None)),
        };
        Ok(terminal)
    }

    fn init_xterm()->Result<XtermImpl>{
        let theme = js_sys::Object::new();
        let theme_opts = Vec::from([
            ("background", JsValue::from("rgba(255,255,255,1)")),
			("foreground", JsValue::from("#000")),
            ("selection", JsValue::from("rgba(0,0,0,0.25)")),
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
            // ("allowTransparency", JsValue::from(false)),
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

    fn init_addons(&self, xterm : &XtermImpl) -> Result<()> {
        log_trace!("Creating FitAddon...");
        let fit = FitAddon::new();
        log_trace!("FitAddon created...");
        xterm.load_addon(fit.clone().into());
        *self.fit.lock().unwrap() = Some(fit);

        Ok(())
    }

    pub async fn init(self : &Arc<Self>, terminal : &Arc<Terminal>)->Result<()>{
        log_trace!("Terminal.init()....");

        let receiver = load_scripts()?;
        receiver.recv().await?;

        let xterm = Self::init_xterm()?;

        self.init_addons(&xterm)?;

        xterm.open(&self.element);

        self.init_kbd_listener(&xterm)?;
        self.init_resize_observer()?;
        self.init_clipboard(&xterm)?;

        *self.xterm.lock().unwrap() = Some(xterm);
        *self.terminal.lock().unwrap() = Some(terminal.clone());
        
        Ok(())
    }

    fn init_clipboard(self : &Arc<Self>, xterm : &XtermImpl) -> Result<()> {

        let this = self.clone();
        let clipboard_listener = Listener::new(move |e:web_sys::KeyboardEvent|->std::result::Result<(), JsValue>{
            //log_trace!("ssssss: key:{}, ctrl_key:{}, meta_key:{},  {:?}", e.key(), e.ctrl_key(), e.meta_key(), e);
            if e.key() == "v" && (e.ctrl_key() || e.meta_key()){
                this.sink.sender.try_send(Ctl::Paste).expect("Unable to send paste Ctl");
            }
            // TODO - detect event type and if paste, paste the content...
            Ok(())
        });
        let mut locked = self.clipboard_listerner.lock().expect("Unable to lock");
        
        // TODO install clipboard handler
        //add_clipboard_event_listener_with_callback("paste", clipboard_listener.into_js());
        xterm.get_element().add_event_listener_with_callback("keydown", clipboard_listener.into_js())?;
        *locked = Some(clipboard_listener);


        Ok(())
    }

    fn init_resize_observer(self : &Arc<Self>) -> Result<()> {
        let this = self.clone();
        let resize_listener = Listener::new(move |_|->std::result::Result<(), JsValue>{
            if let Err(err) = this.resize() {
                log_error!("Resize error: {:?}", err);
            }
            Ok(())
        });
        let resize_observer = ResizeObserver::new(resize_listener.into_js())?;
        resize_observer.observe(&self.element);
        *self.resize.lock().unwrap() = Some((resize_observer,resize_listener));

        Ok(())
    }

    fn init_kbd_listener(self : &Arc<Self>, xterm : &XtermImpl) -> Result<()> {
        let this = self.clone();
        let listener = Listener::new(move |e:XtermEvent|->std::result::Result<(), JsValue>{
            //let term_key = try_get_string(&e, "key")?;
            let term_key = e.get_key();
            let dom_event = e.get_dom_event();
            let key = dom_event.key();
            let ctrl_key = dom_event.ctrl_key();
            let alt_key = dom_event.alt_key();
            let meta_key = dom_event.meta_key();

            //log_trace!("on_key: {:?}, key:{}", e, term_key);
            /*
            let dom_event = try_get_js_value(&e, "domEvent")?;
            let ctrl_key = try_get_bool_from_prop(&dom_event, "ctrlKey").unwrap_or(false);
            let alt_key = try_get_bool_from_prop(&dom_event, "altKey").unwrap_or(false);
            let meta_key = try_get_bool_from_prop(&dom_event, "metaKey").unwrap_or(false);
            */
            //let _key_code = try_get_u64_from_prop(&dom_event, "keyCode")?;
            //let key = try_get_string(&dom_event, "key")?;
            // log_trace!("key_code: {}, key:{}, ctl_key:{}", _key_code, key, ctrl_key);
            this.sink.sender.try_send(
                Ctl::SinkEvent(SinkEvent::new(key, term_key, ctrl_key, alt_key, meta_key))
            ).unwrap();

            Ok(())
        });

        xterm.on_key(listener.into_js());
        *self.listener.lock().unwrap() = Some(listener);

        Ok(())
    }

    pub async fn run(self: &Arc<Self>) -> Result<()> {

        loop {
            let event = self.sink.receiver.recv().await?;
            match event {
                Ctl::SinkEvent(event) => {
                    self.sink(event).await?;
                },
                Ctl::Paste => {
                    //break;
                    let data_js_value = get_clipboard_data().await;
                    if let Some(text) = data_js_value.as_string() {
                        self.terminal
                            .lock()
                            .unwrap()
                            .as_ref()
                            .unwrap()
                            .inject(text)?;
                    }
                }
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
            },
            "Enter" => Key::Enter,
            _=>{
                //log_trace!("ctrl_key:{}, alt_key:{}, meta_key:{}", e.ctrl_key, e.alt_key, e.meta_key);
                let printable = !e.meta_key; // ! (e.ctrl_key || e.alt_key || e.meta_key);
                if !printable{
                    return Ok(());
                }
                //log_trace!("Char: {}", e.key);
                if let Some(c) = e.key.chars().next(){
                    //log_trace!("Char2: {}, {}", e.key, c);
                    //log_trace!("e:{:?}", e);
                    
                    if e.ctrl_key{
                        //log_trace!("ctrl_key:####");
                        /*
                        if c == 'v' {
                            log_trace!("ctrl_key:v");
                            let data = get_clipboard_data().await;
                            log_trace!("data:{:?}", data);
                        }
                        */
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
        
        Ok(())
    }

    pub fn write<S>(&self, s:S) where S:Into<String>{
        self.xterm
            .lock()
            .unwrap()
            .as_ref()
            .expect("Xterm is not initialized")
            .write(s.into());
    }
    // pub fn paste<S>(&self, s:S) where S:Into<String>{
    //     self.xterm
    //         .lock()
    //         .unwrap()
    //         .as_ref()
    //         .expect("Xterm is not initialized")
    //         .paste(s.into());
    // }

    pub fn measure(&self) -> Result<()> {
        // let charSizeService = term._core._charSizeService
        let xterm = self.xterm.lock().unwrap();
        let xterm = xterm.as_ref().unwrap();
        let core = try_get_js_value(xterm, "_core")
            .expect("Unable to get xterm core");
        let char_size_service = try_get_js_value(&core, "_charSizeService")
            .expect("Unable to get xterm charSizeService");
        let has_valid_size = try_get_js_value(&char_size_service, "hasValidSize")
            .expect("Unable to get xterm charSizeService::hasValidSize");

        if has_valid_size.is_falsy() {
            apply_with_args0(&char_size_service, "measure")?;
        }

        Ok(())
    }

    pub fn resize(&self) -> Result<()>{
        self.measure()?;

        // let fit = self.fit.lock().unwrap().as_ref().clone().unwrap();
        let fit = self.fit.lock().unwrap();
        let fit = fit.as_ref().unwrap();
        // FIXME review if this is correct
        fit.propose_dimensions();
        // FIXME review if this is correct
        fit.fit();

		// if(charSizeService && !charSizeService.hasValidSize){
		// 	charSizeService.measure()
		// 	//if(term._core._renderService)
		// 	//	term._core._renderService._updateDimensions();
		// }
		// let addon = this.addons.fit.instance;
		// let dimensions = addon.proposeDimensions()
		// addon.fit();

        Ok(())
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
    inject_blob("xterm.js", Content::Script(xterm_js))?;
    let xterm_addon_fit_js = include_bytes!("../../extern/resources/xterm-addon-fit.js");
    inject_blob("xterm-addon-fit.js",Content::Script(xterm_addon_fit_js))?;
    let xterm_addon_web_links_js = include_bytes!("../../extern/resources/xterm-addon-web-links.js");
    inject_blob_with_callback("xterm-addon-web-links.js",Content::Script(xterm_addon_web_links_js), Some(load))?;
    let xterm_css = include_bytes!("../../extern/resources/xterm.css");
    inject_blob("xterm.css", Content::Style(xterm_css))?;
    inject_css("
        .terminal{
            width:100vw;
            /*border:1px solid #DDD;*/
            height:100vh;
        }
    ")?;
    Ok(())
}



#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (extends = :: js_sys :: Object , js_name = ResizeObserver , typescript_type = "ResizeObserver")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type ResizeObserver;
    #[wasm_bindgen(catch, constructor, js_class = "ResizeObserver")]
    pub fn new(callback: &::js_sys::Function) -> std::result::Result<ResizeObserver, JsValue>;
    #[wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = disconnect)]
    pub fn disconnect(this: &ResizeObserver);
    #[wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = observe)]
    pub fn observe(this: &ResizeObserver, target: &Element);
    // # [wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = observe)]
    // pub fn observe_with_options(
    //     this: &ResizeObserver,
    //     target: &Element,
    //     options: &ResizeObserverOptions,
    // );
    // # [wasm_bindgen (method , structural , js_class = "ResizeObserver" , js_name = unobserve)]
    pub fn unobserve(this: &ResizeObserver, target: &Element);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen (js_namespace=["navigator", "clipboard"], js_name="readText")]
    async fn get_clipboard_data()-> JsValue;
}

