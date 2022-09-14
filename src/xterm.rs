use web_sys::Element;
use workflow_dom::utils::*;
use workflow_log::*;
use crate::Result;
use crate::cli::{Cli, TerminalTrait};
use crate::keys::Key;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::{Mutex, Arc};
use workflow_wasm::listener::Listener;
use workflow_wasm::utils::*;

/// #[wasm_bindgen(module = "/defined-in-js.js")]
#[wasm_bindgen()]
extern "C" {

    #[wasm_bindgen(js_namespace=window, js_name="Terminal")]
    type Term;

    #[wasm_bindgen(constructor, js_class = "Terminal")]
    fn new(opt: js_sys::Object) -> Term;

    #[wasm_bindgen(method, getter)]
    fn number(this: &Term) -> u32;

    #[wasm_bindgen(method)]
    fn open(this: &Term, el: &Element);

    #[wasm_bindgen(method, js_name="onKey")]
    fn on_key(this: &Term, f: &js_sys::Function);

    #[wasm_bindgen(method, js_name="write")]
    fn _write(this: &Term, text:String);
}

impl Debug for Term{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "TODO::")?;
        Ok(())
    }
}

impl Term{
    fn write<T:Into<String>>(&self, text:T){
        self._write(text.into());
    }
}

pub struct Terminal{
    pub element: Element,
    term:Term,
    listener: Arc<Mutex<Option<Listener<JsValue>>>>,
    cli: Arc<Mutex<Option<Cli>>>
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
        Self{
            key,
            term_key,
            ctrl_key,
            alt_key,
            meta_key, 
        }
    }
}

impl Terminal{
    pub fn new(parent:&Element)->Result<Arc<Terminal>>{
        let element = document().create_element("div")?;
        element.set_attribute("class", "terminal")?;
        parent.append_child(&element)?;
        let terminal = Terminal{
            element,
            listener: Arc::new(Mutex::new(None)),
            term: Self::create_term()?,
            cli: Arc::new(Mutex::new(None))
        };
        let term = terminal.init()?;
        Ok(term)
    }

    fn create_term()->Result<Term>{
        let theme = js_sys::Object::new();
        let theme_opts = Vec::from([
            ("background", JsValue::from("rgba(0,0,0,1)")),
			("foreground", JsValue::from("#F0F")),
			("cursor", JsValue::from("#F00"))
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
        
        let term = Term::new(options);
        log_trace!("term: {:?}", term);

        Ok(term)
    }

    pub fn init(self)->Result<Arc<Self>>{
        log_trace!("Terminal.init()....");

        self.term.open(&self.element);


        let self_arc = Arc::new(self);
        let this = self_arc.clone();
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
            this.sink(SinkEvent::new(key, term_key, ctrl_key, alt_key, meta_key), e)?;
            
            Ok(())
        });

        self_arc.term.on_key(listener.into_js());
        let self_arc_clone = self_arc.clone();
        
        let mut locked_listener =  self_arc.listener.lock().expect("Unable to lock terminal listener");
        *locked_listener = Some(listener);

        Ok(self_arc_clone)
    }

    fn sink(self: &Arc<Self>, e:SinkEvent, _e:JsValue)->Result<()>{

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

        let mut locked = self.cli.lock().expect("Unable to lock terminal.cli for intake");
        log_trace!("e3:{:?}", e);
        if let Some(cli) = locked.as_mut(){
            log_trace!("cli.intake: {:?}, {}", key, e.term_key);
            cli.intake(key, e.term_key)?;
        }

        Ok(())
    }
  
}

unsafe impl Send for Terminal{}
unsafe impl Sync for Terminal{}

impl TerminalTrait for Terminal{
    fn write(&self, s: String) -> Result<()> {
        self.term.write(s);
        Ok(())
    }

    fn input_handler(&self, cli:Cli)-> Result<()> {
        let mut locked = self.cli.lock().expect("Unable to lock terminal.cli");
        *locked = Some(cli);
        Ok(())
    }

    fn start(&self)-> Result<()> {
        //self._start()?;
        Ok(())
    }
}