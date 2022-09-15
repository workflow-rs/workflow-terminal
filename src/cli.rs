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
    buffer:Vec<String>,
    history:Vec<Vec<String>>,
    cursor:usize,
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
    //fn write<S>(&self, s: S) -> Result<()> where S:Into<String>;
    //fn input_handler(&self, h:Cli)-> Result<()>;
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


#[derive(Clone)]
pub struct Cli {
    pub inner : Arc<Mutex<Inner>>,
    running: Arc<AtomicBool>,
    term : Arc<dyn Terminal>,
    prompt : Arc<Mutex<String>>,
}

impl Cli {

    pub fn new(
        term : Arc<dyn Terminal>,
        prompt : Arc<Mutex<String>>,
    ) -> Result<Cli> {

        let cli = Cli {
            inner : Arc::new(Mutex::new(Inner::new())),
            running : Arc::new(AtomicBool::new(false)),
            term,
            prompt,
        };

        cli.init()?;

        Ok(cli)
    }

    fn init(&self)->Result<()>{
        /*
        let this = self.clone();
        / *
        self.term.input_handler(Box::new(move |e:TerminalEvent|->Result<()>{
            this.intake(e.key, e.key_str);
            Ok(())
        }))?;
        * /
        self.term.input_handler(this)?;
        */
        Ok(())
    }

    pub fn start(&self)->Result<()>{
        self.term.start()?;
        Ok(())
    }

    pub fn inner(&self) -> LockResult<MutexGuard<'_, Inner>> {
        self.inner.lock()
    }

    fn write_vec(&self, mut str_list:Vec<String>) ->Result<()> {
        let data = self.inner()?;
		
        str_list.push("\r\n".to_string());
        
		if self.running.load(Ordering::SeqCst) {
			self.term.write(str_list.join(""))?;
		}else {
			self.term.write(format!("\x1B[2K\r{}", str_list.join("")))?;
			let prompt = format!("{}{}", self.prompt_str(), data.buffer.join(""));
			self.term.write(prompt)?;
			let l = data.buffer.len() - data.cursor;
			for _ in 0..l{
				self.term.write("\x08".to_string())?;
            }
		}

        Ok(())
	}

    fn _write<S>(&self, s : S)->Result<()> where S : Into<String> {
        self.term.write(s.into())?;
        Ok(())
    }

	pub fn write<S>(&self, s : S)-> Result<()>  where S : Into<String> {
        let s:String = s.into();
		self.write_vec(Vec::from([s]))?;
        Ok(())
	}

    // pub fn term(&self) -> LockResult<MutexGuard<'_, Arc<dyn Terminal>>> {
    //     self.term.lock()
    // }

    fn prompt_str(&self) -> String {
        return self.prompt.lock().unwrap().clone();
    }

    /*
    pub fn _prompt(&self, data:&mut MutexGuard<Inner>) -> Result<()> {
		data.cursor = 0;
		data.buffer = Vec::new();

		self.term.write(format!("\r\n{}", self.prompt_str()))?;

        Ok(())
	}
    */

    pub fn prompt(&self) -> Result<()> {
        /*
        let mut data = self.inner()?;
		self._prompt(&mut data)?;
        */
        let mut data = self.inner()?;
        data.cursor = 0;
		data.buffer = Vec::new();

        log_trace!("prompt...");

		self.term.write(format!("\r\n{}", self.prompt_str()))?;
        Ok(())
	}



    fn inject(&self, term_key : String) -> Result<()> {
        let mut data = self.inner()?;
        let mut vec = data.buffer.clone();
        let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), [term_key]).collect();
        data.buffer = vec;
        //log_trace!("inject: data.buffer: {:#?}", data.buffer);
        //log_trace!("inject: removed: {:#?}", removed);
        self.trail(data.cursor, &data.buffer, true, false, 1)?;

        data.cursor = data.cursor+1;
        Ok(())
    }

    pub fn intake(&self, key : Key, _term_key : String) -> Result<()> {
        let running = self.running.load(Ordering::SeqCst);
        match key {
            Key::Backspace => {
                if running{ return Ok(()); }
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                self._write("\x08")?;
                data.cursor = data.cursor - 1;
                let mut vec = data.buffer.clone();
                vec.splice(data.cursor..(data.cursor+1), []);
                data.buffer = vec;
                self.trail(data.cursor, &data.buffer, true, true, 0)?;
            },
            Key::ArrowUp =>{
                if running{ return Ok(()); }
                let mut data = self.inner()?;
                if data.history_index == 0{
                    return Ok(());
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
                self._write(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")))?;
                data.cursor = data.buffer.len();
                
            }
            Key::ArrowDown =>{
                if running{ return Ok(()); }
                let mut data = self.inner()?;
                let len =  data.history.len();
                if data.history_index >= len{
                    return Ok(());
                }
                let index = data.history_index;
                data.history[index] = data.buffer.clone();
                data.history_index = data.history_index+1;
                if data.history_index == len{
                    data.buffer = Vec::new();
                }else{
                    data.buffer = data.history[data.history_index].clone();
                }
                
                self._write(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")))?;
                data.cursor = data.buffer.len();
            }
            Key::ArrowLeft =>{
                if running{ return Ok(()); }
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                data.cursor = data.cursor-1;
                self._write(Left(1))?;
            }
            Key::ArrowRight =>{
                if running{ return Ok(()); }
                let mut data = self.inner()?;
                if data.cursor < data.buffer.len() {
                    data.cursor = data.cursor+1;
                    self._write(Right(1))?;
                }
            }
            // "Inject"=>{
            //     inject(term_key);
            // }
            Key::Enter => {
                if running{ return Ok(()); }
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
                    self._write("\r\n")?;
                    self.running.store(true, Ordering::SeqCst);
                    self.digest(cmd)?;
                    //#[cfg(not(target_arch="wasm32"))]
                    //self.after_digest()?;
                }else{
                    self.prompt()?;
                }
            },
            Key::Alt(_c)=>{
                if running{ return Ok(()); }
                return Ok(());
            },
            Key::Ctrl(_c)=>{
                return Ok(());
            },
            Key::Char(ch)=>{
                if running{ return Ok(()); }
                self.inject(ch.to_string())?;
            },
            _ => {
                return Ok(());
            }
        }

        Ok(())
    }

    pub fn after_digest(&self)-> Result<()> {
        self.running.store(false, Ordering::SeqCst);
        self.prompt()?;
        Ok(())
    }

    fn trail(&self, cursor:usize, buffer:&Vec<String>, rewind: bool, erase_last : bool, offset : usize) ->Result<()>{
		let mut tail = buffer[cursor..].join("");
        if erase_last{
            tail = tail+" ";
        }
		self._write(&tail)?;
        if rewind{
            let mut l = tail.len();
            if offset > 0{
                l = l-offset;
            }
            for _ in 0..l{
                self._write("\x08")?;//backspace
            }
        }
        Ok(())
	}

    /*
    fn build_arg(&self, s:&str)->String{
        let s = s.trim();
        if (s.starts_with("\"") && s.ends_with("\"")) 
            || (s.starts_with("'") && s.ends_with("'"))
        {
            return s.to_string();
        }

        s.replace("  ", " ")
    }
    */

    fn digest(&self, cmd: String) -> Result<()> {
        /*
        let mut args:Vec<String> = cmd.split(" ").map(|a|{
            self.build_arg(a)
        }).filter(|a|{a.len()>0}).collect();
        let cmd = args.remove(0);
        */
        
        /*
        let this = self.clone();
        spawn(async move {
            log_trace!("Digesting: {}", cmd);
            let locked = this.handler.lock().expect("Unable to unlock handler");
            let _r = locked.clone().digest(cmd).await;
        });
        */
        self.term.digest(cmd)?;
        Ok(())
    }


}
