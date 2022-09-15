use web_sys::Element;
use workflow_dom::utils::*;
use workflow_log::*;
use crate::Result;
use crate::cli::{Intake, Terminal as TerminalTrait, CliHandler, DefaultHandler};
use crate::keys::Key;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::{Mutex, Arc};
use workflow_wasm::listener::Listener;
use workflow_wasm::utils::{*, self};

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


pub struct Options{
    pub prompt:String
}

pub struct Terminal{
    pub element: Element,
    term:Term,
    listener: Arc<Mutex<Option<Listener<JsValue>>>>,
    intake: Arc<Intake>,
    handler: Arc<Mutex<Arc<dyn CliHandler>>>,
    //cli: Option<Cli>
}

impl Terminal{
    pub fn new(parent:&Element, opt:Options)->Result<Arc<Terminal>>{
        let element = document().create_element("div")?;
        element.set_attribute("class", "terminal")?;
        parent.append_child(&element)?;
        let terminal = Terminal{
            element,
            listener: Arc::new(Mutex::new(None)),
            term: Self::create_term()?,
            intake: Arc::new(Intake::new(Arc::new(Mutex::new(opt.prompt)))?),
            handler: Arc::new(Mutex::new(Arc::new(DefaultHandler::new()))),
            //cli: None
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
            //let locked = this.lock().expect("msg");
            this.sink(SinkEvent::new(key, term_key, ctrl_key, alt_key, meta_key), e)?;
            
            Ok(())
        });
        //let locked = self_arc.lock().expect("Unable to lock terminal");
        self_arc.term.on_key(listener.into_js());
        let self_arc_clone = self_arc.clone();
        
        let mut locked_listener =  self_arc.listener.lock().expect("Unable to lock terminal listener");
        *locked_listener = Some(listener);

        Ok(self_arc_clone)
    }

    fn sink(&self, e:SinkEvent, _e:JsValue)->Result<()>{

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

        
        let res = self.intake.process_key(key, e.term_key)?;
        
        for text in res.texts{
            self.term.write(text);
        }
        if let Some(cmd) = res.cmd{
            self.digest(cmd)?;
        }

        /*
        if let Some(cli) = self.cli.as_ref(){
            cli.intake(key, e.term_key)?;
        }
        */


        Ok(())
    }

    pub fn write_str<S>(&self, text:S)->Result<()> where S:Into<String>{
        self.term.write(text.into());
        Ok(())
    }

    pub fn prompt(&self)->Result<()>{
        self.term.write(self.intake.prompt()?);
        Ok(())
    }


    fn write_vec(&self, mut str_list:Vec<String>) ->Result<()> {
        let data = self.intake.inner()?;
		
        str_list.push("\r\n".to_string());
        
		if self.intake.is_running() {
			self.term.write(str_list.join(""));
		}else {
			self.term.write(format!("\x1B[2K\r{}", str_list.join("")));
			let prompt = format!("{}{}", self.intake.prompt_str(), data.buffer.join(""));
			self.term.write(prompt);
			let l = data.buffer.len() - data.cursor;
			for _ in 0..l{
				self.term.write("\x08".to_string());
            }
		}

        Ok(())
	}

    fn _write<S>(&self, s : S)->Result<()> where S : Into<String> {
        self.term.write(s.into());
        Ok(())
    }

	/*
    fn write<S>(&self, s : S)-> Result<()>  where S : Into<String> {
        let s:String = s.into();
		self.write_vec(Vec::from([s]))?;
        Ok(())
	}
    */

  
}

unsafe impl Send for Terminal{}
unsafe impl Sync for Terminal{}

impl TerminalTrait for Terminal{
    fn write(&self, s: String) -> Result<()>{
        //self.term.write(s);
        self.write_vec(Vec::from([s]))?;
        Ok(())
    }

    fn start(&self)-> Result<()> {
        //self._start()?;
        Ok(())
    }

    fn digest(&self, cmd: String) -> Result<()>{
        let hander = self.handler.clone();
        let intake = self.intake.clone();
        let term = self.term.clone();
        crate::spawn(async move{
            let locked = hander.lock().expect("Unable to lock terminal.handler for digest");
            let _r = locked.digest(cmd).await;
            match intake.after_digest(){
                Ok(text)=>{
                    let _r = utils::apply_with_args1(&term, "write", JsValue::from(text));
                }
                Err(_e)=>{
                    //
                }
            }
        });
        Ok(())
    }

    fn register_handler(&self, hander: Arc<dyn CliHandler>)-> Result<()> {
        let mut locked = self.handler.lock().expect("Unable to lock terminal.handler");
        *locked = hander;
        Ok(())
    }
}
