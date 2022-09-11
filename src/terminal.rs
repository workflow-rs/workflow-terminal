use web_sys::Element;
use workflow_dom::utils::*;
use workflow_log::*;
use crate::Result;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;
use std::fmt::Debug;
use std::fmt::Formatter;
// use crate::Listener;
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

pub struct Data{
    buffer:Vec<String>,
    history:Vec<Vec<String>>,
    cursor:usize,
    history_index:usize,
    running:bool
}
impl Data{
    fn new()->Self{
        Self{
            buffer:Vec::new(),
            history:Vec::new(),
            cursor:0,
            history_index:0,
            running:false
        }
    }
}

pub struct Terminal{
    pub element: Element,
    term:Term,
    prompt_prefix:String,
    listener: Arc<Mutex<Option<Listener<JsValue>>>>,
    data: Arc<Mutex<Data>>
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
            data: Arc::new(Mutex::new(Data::new())),
            prompt_prefix:"$".to_string(),
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
        //let terminal_js = utils::js_value(&window(), "Terminal")?;
        //log_trace!("terminal_js: {:?}", terminal_js);

        self.term.open(&self.element);


        let self_arc = Arc::new(self);
        let this = self_arc.clone();
        let listener = Listener::new(move |e|->Result<()>{
            let term_key = try_get_string(&e, "key")?;
            //log_trace!("on_key: {:?}, key:{}", e, key);
            let dom_event = try_get_js_value(&e, "domEvent")?;
            let key_code = try_get_u64_from_prop(&dom_event, "keyCode")?;
            let key = try_get_string(&dom_event, "key")?;
            log_trace!("key_code: {}, key:{}", key_code, key);
            //term_.write(key);
            this.sink(key, term_key, e)?;
            
            Ok(())
        });

        self_arc.term.on_key(listener.into_js());
        let self_arc_clone = self_arc.clone();
        //let mut locked = self_arc.lock().expect("Unable to lock term");
        
        let mut locked_listener =  self_arc.listener.lock().expect("Unable to lock terminal listener");
        *locked_listener = Some(listener);

        self_arc_clone.prompt();

        //let args = Array::new_with_length(1);
        //args.set(0, JsValue::from(options));
        
        /*
        let term = js_sys::Reflect::construct(&terminal_js.into(), &args)?;
        log_trace!("term: {:?}", term);

        utils::apply_with_args1(&term, "open", JsValue::from(&self.element))?;
        */

        /*
        let term = new Terminal({
			allowTransparency: true,
			fontFamily: this['font-family'] || 'Consolas, Ubuntu Mono, courier-new, courier, monospace',
			fontSize: this['font-size'] || 20,
			cursorBlink : true,
			theme: {
				background: this.background || 'rgba(0,0,0,0.0)',
				foreground: this.foreground || '#000000',
				cursor: this.cursor || this.foreground || "#FFF"
			}
		});
        */
    	//this.term = term;

        Ok(self_arc_clone)
    }

    fn sink(self: &Arc<Self>, key:String, term_key:String, _e:JsValue)->Result<()>{
        let mut data = self.data.lock().expect("Unable to lock terminal listener");
        //buffer.push(key.clone());
        //let term = self.term.as_ref().ok_or("term not found")?;
        
        let mut handle_key = |key:String, term_key:String|{
            let mut inject = |term_key:String|{
                let mut vec = data.buffer.clone();
                log_trace!("inject: vec: {}", vec.join(""));
                let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), [term_key]).collect();
                data.buffer = vec;
                //log_trace!("inject: data.buffer: {:#?}", data.buffer);
                //log_trace!("inject: removed: {:#?}", removed);
                self.trail(data.cursor, &data.buffer, (true, false, 1));
                data.cursor = data.cursor+1;
            };
            match key.as_str(){
                "Backspace" => {
                    if data.cursor == 0{
                        return;
                    }
                    self.term.write("\x08");
                    data.cursor = data.cursor - 1;
                    let mut vec = data.buffer.clone();
                    vec.splice(data.cursor..(data.cursor+1), []);
                    data.buffer = vec;
                    self.trail(data.cursor, &data.buffer, (true, true, 0));
                },
                "ArrowUp"=>{
                    //log_trace!("ArrowUp");
                    if data.history_index == 0{
                        return;
                    }
                    let current_buffer = data.buffer.clone();
                    let index = data.history_index;
                    log_trace!("ArrowUp: index {}, data.history.len(): {}", index, data.history.len());
                    if data.history.len() <= index{
                        data.history.push(current_buffer);
                    }else{
                        data.history[index] = current_buffer;
                    }
                    data.history_index = data.history_index-1;
                    
                    data.buffer = data.history[data.history_index].clone();
                    self.term.write(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")));
                    data.cursor = data.buffer.len();
                    
                }
                "ArrowDown"=>{
                    //log_trace!("ArrowDown");
                    let len =  data.history.len();
                    if data.history_index >= len{
                        return;
                    }
                    let index = data.history_index;
                    data.history[index] = data.buffer.clone();
                    data.history_index = data.history_index+1;
                    if data.history_index == len{
                        data.buffer = Vec::new();
                    }else{
                        data.buffer = data.history[data.history_index].clone();
                    }
                    
                    self.term.write(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")));
                    data.cursor = data.buffer.len();
                }
                "ArrowLeft"=>{
                    //log_trace!("ArrowLeft");
                    if data.cursor == 0{
                        return;
                    }
                    data.cursor = data.cursor-1;
                    self.term.write(term_key);
                }
                "ArrowRight"=>{
                    //log_trace!("ArrowRight");
                    if data.cursor < data.buffer.len() {
                        data.cursor = data.cursor+1;
                        self.term.write(term_key);
                    }
                }
                "Inject"=>{
                    inject(term_key);
                }
                "Enter" => {
                    //e.stopPropagation();
                    //let { buffer, history } = this;
                    //let { length } = history;
                    let buffer = data.buffer.clone();
                    let length = data.history.len();
    
                    self.term.write("\r\n");
                    data.buffer = Vec::new();
                    data.cursor = 0;
    
                    if buffer.len() > 0 {
                        
                        let cmd = buffer.join("");
                        if length==0 || data.history[length-1].len() > 0{
                            data.history_index = length;
                        }else{
                            data.history_index = length-1;
                        }
                        let index = data.history_index;
                        //log_trace!("length:{length},  history_index:{index}");
                        if length<=index {
                            data.history.push(buffer);
                        }else{
                            data.history[index] = buffer;
                        }
                        data.history_index = data.history_index+1;

                        //log_trace!("length222:{length},  history_index:{}", data.history_index);
    
                        data.running = true;
                        self.digest(cmd);
                        data.running = false;
                    }
    
                    self._prompt(&mut data);
                },
                _=>{
                    let printable = true;//TODO
                    if printable {
                        inject(term_key);
                    }
                }
            }
        };

        handle_key(key, term_key);
        
        //term.write(&key);



        Ok(())
    }

    fn digest(self:&Arc<Self>, cmd:String){
        log_trace!("digest cmd: {cmd}");
    }

    fn prompt_str(&self) -> String{
		format!("{} ", self.prompt_prefix)
	}

    fn _prompt(&self, data:&mut Data){
		data.cursor = 0;
		data.buffer = Vec::new();

		self.term.write(format!("\r\n{}", self.prompt_str()));
		//self.promptLength = ("\r\n"+prompt).length;
	}

    pub fn prompt(&self){
        let mut data = self.data.lock().expect("Unable to lock terminal listener");
		self._prompt(&mut data);
	}

    pub fn write_vec(&self, mut str_list:Vec<String>) {
        let data = self.data.lock().expect("Unable to lock terminal listener");
		
        str_list.push("\r\n".to_string());
        
		if data.running {
			self.term.write(str_list.join(""));
		}else {
			self.term.write(format!("\x1B[2K\r{}", str_list.join("")));
			let prompt = format!("{}{}", self.prompt_str(), data.buffer.join(""));
			self.term.write(prompt);
			let l = data.buffer.len() - data.cursor;
			for _ in 0..l{
				self.term.write("\x08");
            }
		}
	}
	pub fn write_str<T:Into<String>>(&self, str:T){
        let s:String = str.into();
		self.write_vec(Vec::from([s]));
	}

    
    fn trail(self: &Arc<Self>, x:usize, buffer:&Vec<String>, options:(bool, bool, usize)) {
		let (rewind, erase_last, offset) = options;
		let mut tail = buffer[x..].join("");
        if erase_last{
            tail = tail+" ";
        }
		self.term.write(&tail);
        if rewind{
            let mut l = tail.len();
            if offset > 0{
                l = l-offset;
            }
            for _ in 0..l{
                self.term.write("\x08");//backspace
            }
        }
	}
}