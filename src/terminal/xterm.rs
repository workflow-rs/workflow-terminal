use workflow_dom::inject::*;
use web_sys::Element;
use workflow_dom::utils::*;
use workflow_log::*;
use crate::Result;
use crate::keys::Key;
use crate::terminal::Terminal;
use crate::terminal::Options;
use crate::terminal::TargetElement;
use wasm_bindgen::JsValue;
use std::fmt::Debug;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Mutex, Arc};
use workflow_wasm::listener::Listener;
use workflow_wasm::utils::*;
use workflow_core::channel::{oneshot,unbounded,Sender,Receiver};
use workflow_dom::utils::body;
use wasm_bindgen::prelude::*;
use super::bindings::*;
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
    _web_links : Arc<Mutex<Option<WebLinksAddon>>>,
    clipboard_listerner:  Arc<Mutex<Option<Listener<web_sys::KeyboardEvent>>>>,
    terminate : Arc<AtomicBool>,
}

impl Xterm{

    pub fn try_new() -> Result<Self> {
        Self::new_with_options(Options::default())
    }

    pub fn new_with_options(options: Options) -> Result<Self> {
        let el = match &options.element {
            TargetElement::Body => {
                body().expect("Unable to get 'body' element")
            },
            TargetElement::Element(el) => el.clone(),
            TargetElement::TagName(tag) => {
                document()
                    .get_elements_by_tag_name(&tag)
                    .item(0)
                    .ok_or("Unable to locate parent element for terminal")?
            },
            TargetElement::Id(id) => {
                document()
                    .get_element_by_id(&id)
                    .ok_or("Unable to locate parent element for terminal")?
            }
        };
        Self::new_with_element(&el, options)
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
            _web_links : Arc::new(Mutex::new(None)),
            clipboard_listerner: Arc::new(Mutex::new(None)),
            terminate : Arc::new(AtomicBool::new(false)),
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
        let fit = FitAddon::new();
        xterm.load_addon(fit.clone().into());
        *self.fit.lock().unwrap() = Some(fit);
        Ok(())
    }

    pub async fn init(self : &Arc<Self>, terminal : &Arc<Terminal>)->Result<()>{

        let receiver = load_scripts()?;
        receiver.recv().await?;

        let xterm = Self::init_xterm()?;

        self.init_addons(&xterm)?;

        xterm.open(&self.element);
        xterm.focus();

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
            //log_trace!("xterm: key:{}, ctrl_key:{}, meta_key:{},  {:?}", e.key(), e.ctrl_key(), e.meta_key(), e);
            if e.key() == "v" && (e.ctrl_key() || e.meta_key()){
                this.sink.sender.try_send(Ctl::Paste).expect("Unable to send paste Ctl");
            }
            Ok(())
        });
        let mut locked = self.clipboard_listerner.lock().expect("Unable to lock");
        
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

            this.sink.sender.try_send(
                Ctl::SinkEvent(SinkEvent::new(key, term_key, ctrl_key, alt_key, meta_key))
            ).unwrap();

            Ok(())
        });

        xterm.on_key(listener.into_js());
        *self.listener.lock().unwrap() = Some(listener);

        Ok(())
    }

    pub fn terminal(&self) -> Arc<Terminal> {
        self.terminal.lock().unwrap().as_ref().unwrap().clone()
    }

    pub async fn run(self: &Arc<Self>) -> Result<()> {
        self.intake(&self.terminate).await?;
        Ok(())
    }

    pub async fn intake(self: &Arc<Self>, terminate : &Arc<AtomicBool>) -> Result<()> {
        loop {
            if terminate.load(Ordering::SeqCst) {
                break;
            }
            
            let event = self.sink.receiver.recv().await?;
            match event {
                Ctl::SinkEvent(event) => {
                    self.sink(event).await?;
                },
                Ctl::Paste => {
                    //break;
                    let data_js_value = get_clipboard_data().await;
                    if let Some(text) = data_js_value.as_string() {
                        self.terminal().inject(text)?;
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
        self.terminate.store(true, Ordering::SeqCst);
        self.sink.sender.try_send(Ctl::Close).expect("Unable to send exit Ctl");
    }

    async fn sink(&self, e:SinkEvent)->Result<()>{

        let key = match e.key.as_str(){
            "Backspace" => Key::Backspace,
            "ArrowUp"=> Key::ArrowUp,
            "ArrowDown"=> Key::ArrowDown,
            "ArrowLeft"=> Key::ArrowLeft,
            "ArrowRight"=>Key::ArrowRight,
            "Escape"=>Key::Esc,
            "Delete"=>Key::Delete,
            "Tab"=>{
                // TODO implement completion handler
                return Ok(());
            },
            "Enter" => Key::Enter,
            _=>{
                let printable = !e.meta_key; // ! (e.ctrl_key || e.alt_key || e.meta_key);
                if !printable{
                    return Ok(());
                }
                //log_trace!("e:{:?}", e);
                if let Some(c) = e.key.chars().next() {
                    if e.ctrl_key {
                        Key::Ctrl(c)
                    } else {
                        if e.alt_key {
                            Key::Alt(c)
                        } else {
                            Key::Char(c)
                        }
                    }
                } else {
                    return Ok(());
                }
            }
        };

        self.terminal().ingest(key, e.term_key).await?;
        
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

    pub fn measure(&self) -> Result<()> {
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

        let fit = self.fit.lock().unwrap();
        let fit = fit.as_ref().unwrap();
        // TODO review if this is correct
        fit.propose_dimensions();
        // TODO review if this is correct
        fit.fit();

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
            height:100vh;
        }
    ")?;
    Ok(())
}

