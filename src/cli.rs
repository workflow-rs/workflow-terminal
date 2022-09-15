use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, LockResult, atomic::AtomicBool};
//use workflow_log::*;
use crate::result::Result;
use crate::keys::Key;
use crate::cursor::*;
//use crate::spawn;
use async_trait::async_trait;
use workflow_log::log_trace;


#[derive(Debug)]
pub struct Inner {
    pub buffer:Vec<String>,
    history:Vec<Vec<String>>,
    pub cursor:usize,
    history_index:usize,
}

impl Inner {
    pub fn new() -> Self {
        Inner {
            buffer:Vec::new(),
            history:Vec::new(),
            cursor:0,
            history_index:0,
        }
    }
}


pub trait Terminal : Sync + Send {
    fn write(&self, s: String) -> Result<()>;
    fn start(&self)-> Result<()>;
    fn digest(&self, cmd: String) -> Result<()>;
    fn register_handler(&self, hander: Arc<dyn CliHandler>)-> Result<()>;
}

#[async_trait]
pub trait CliHandler : Sync + Send{
    async fn digest(&self, cmd: String) -> Result<()>;
    async fn complete(&self, substring : String) -> Result<Vec<String>>;
}
pub struct DefaultHandler{}

impl DefaultHandler{
    pub fn new()->Self{
        Self{}
    }
}

#[async_trait]
impl CliHandler for DefaultHandler{
    async fn digest(&self, _cmd:String)->Result<()>{
        Ok(())
    }

    async fn complete(&self, substring : String) -> Result<Vec<String>> {
        if substring.starts_with('a') {
            Ok(vec!["alpha".to_string(), "aloha".to_string(), "albatross".to_string()])
        } else {
            Ok(vec![])
        }
    }
}

pub struct ProcessResult{
    pub texts: Vec<String>,
    pub cmd: Option<String>
}

impl ProcessResult{
    fn empty()->Self{
        Self{texts:Vec::new(), cmd:None}
    }
    fn new(texts: Vec<String>)->Self{
        Self{texts, cmd:None}
    }
    fn new_with_cmd(texts: Vec<String>, cmd:String)->Self{
        Self{texts, cmd:Some(cmd)}
    }
}


#[derive(Clone)]
pub struct Intake {
    pub inner : Arc<Mutex<Inner>>,
    pub running: Arc<AtomicBool>,
    pub prompt : Arc<Mutex<String>>,
}

impl Intake {

    pub fn new(
        prompt : Arc<Mutex<String>>,
    ) -> Result<Self> {

        let intake = Self {
            inner : Arc::new(Mutex::new(Inner::new())),
            running : Arc::new(AtomicBool::new(false)),
            prompt,
        };

        intake.init()?;

        Ok(intake)
    }

    fn init(&self)->Result<()>{
        Ok(())
    }

    pub fn inner(&self) -> LockResult<MutexGuard<'_, Inner>> {
        self.inner.lock()
    }

    pub fn prompt_str(&self) -> String {
        return self.prompt.lock().unwrap().clone();
    }

    pub fn prompt(&self) -> Result<String> {
        /*
        let mut data = self.inner()?;
		self._prompt(&mut data)?;
        */
        let mut data = self.inner()?;
        data.cursor = 0;
		data.buffer = Vec::new();

        //log_trace!("prompt...");

		Ok(format!("\r\n{}", self.prompt_str()))
	}



    fn inject(&self, term_key : String) -> Result<String> {
        let mut data = self.inner()?;
        let mut vec = data.buffer.clone();
        let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), [term_key]).collect();
        data.buffer = vec;
        //log_trace!("inject: data.buffer: {:#?}", data.buffer);
        //log_trace!("inject: removed: {:#?}", removed);
        let texts = self.trail(data.cursor, &data.buffer, true, false, 1)?;

        data.cursor = data.cursor+1;
        Ok(texts)
    }

    pub fn process_key(&self, key : Key, _term_key : String) -> Result<ProcessResult> {
        let running = self.running.load(Ordering::SeqCst);
        let mut texts:Vec<String> = Vec::new();
        fn empty()->Result<ProcessResult>{
            Ok(ProcessResult::empty())
        }
        match key {
            Key::Backspace => {
                if running{ return empty(); }
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return empty();
                }
                texts.push("\x08".to_string());
                data.cursor = data.cursor - 1;
                let mut vec = data.buffer.clone();
                vec.splice(data.cursor..(data.cursor+1), []);
                data.buffer = vec;
                texts.push(self.trail(data.cursor, &data.buffer, true, true, 0)?);
            },
            Key::ArrowUp =>{
                if running{ return empty(); }
                let mut data = self.inner()?;
                if data.history_index == 0{
                    return empty();
                }
                let current_buffer = data.buffer.clone();
                let index = data.history_index;
                //log_trace!("ArrowUp: index {}, data.history.len(): {}", index, data.history.len());
                if data.history.len() <= index{
                    data.history.push(current_buffer);
                }else{
                    data.history[index] = current_buffer;
                }
                data.history_index = data.history_index-1;
                
                data.buffer = data.history[data.history_index].clone();
                texts.push(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")));
                data.cursor = data.buffer.len();
                
            }
            Key::ArrowDown =>{
                if running{ return empty(); }
                let mut data = self.inner()?;
                let len =  data.history.len();
                if data.history_index >= len{
                    return empty();
                }
                let index = data.history_index;
                data.history[index] = data.buffer.clone();
                data.history_index = data.history_index+1;
                if data.history_index == len{
                    data.buffer = Vec::new();
                }else{
                    data.buffer = data.history[data.history_index].clone();
                }
                
                texts.push(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")));
                data.cursor = data.buffer.len();
            }
            Key::ArrowLeft =>{
                if running{ return empty(); }
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return empty();
                }
                data.cursor = data.cursor-1;
                texts.push(Left(1).to_string());
            }
            Key::ArrowRight =>{
                if running{ return empty(); }
                let mut data = self.inner()?;
                if data.cursor < data.buffer.len() {
                    data.cursor = data.cursor+1;
                    texts.push(Right(1).to_string());
                }
            }
            // "Inject"=>{
            //     inject(term_key);
            // }
            Key::Enter => {
                if running{ return empty(); }
                //log_trace!("Key::Enter:cli");
                let cmd = {
                    let mut data = self.inner()?;
                    //e.stopPropagation();
                    //let { buffer, history } = this;
                    //let { length } = history;
                    let buffer = data.buffer.clone();
                    let length = data.history.len();

                    
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

                        //log_trace!("length222:{length},  history_index:{}, {}", data.history_index, cmd);
                        Some(cmd)
                    } else {
                        None
                    }
                };

                if let Some(cmd) = cmd {
                    texts.push("\r\n".to_string());
                    self.running.store(true, Ordering::SeqCst);
                    return Ok(ProcessResult::new_with_cmd(texts, cmd));
                    //self.digest(cmd)?;
                    //#[cfg(not(target_arch="wasm32"))]
                    //self.after_digest()?;
                }else{
                    texts.push(self.prompt()?);
                }
            },
            Key::Alt(_c)=>{
                if running{ return empty(); }
                return empty();
            },
            Key::Ctrl(_c)=>{
                return empty();
            },
            Key::Char(ch)=>{
                if running{ return empty(); }
                texts.push(self.inject(ch.to_string())?);
            },
            _ => {
                return empty();
            }
        }

        return Ok(ProcessResult::new(texts));
    }

    pub fn after_digest(&self)-> Result<String> {
        self.running.store(false, Ordering::SeqCst);
        let text = self.prompt()?;
        Ok(text)
    }

    fn trail(&self, cursor:usize, buffer:&Vec<String>, rewind: bool, erase_last : bool, offset : usize) ->Result<String>{
		let mut texts = Vec::new();
        let mut tail = buffer[cursor..].join("");
        if erase_last{
            tail = tail+" ";
        }
		texts.push(tail.clone());
        if rewind{
            let mut l = tail.len();
            if offset > 0{
                l = l-offset;
            }
            for _ in 0..l{
                texts.push("\x08".to_string());//backspace
            }
        }
        Ok(texts.join(""))
	}

    pub fn is_running(&self)->bool{
        self.running.load(Ordering::SeqCst)
    }
}
