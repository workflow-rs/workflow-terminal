use cfg_if::cfg_if;
use regex::Regex;
//use workflow_log::log_trace;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, LockResult, atomic::AtomicBool};
use workflow_core::channel::{unbounded,Sender,Receiver};
use crate::result::Result;
use crate::result::CliResult;
use crate::keys::Key;
use crate::cursor::*;
use workflow_log::*;
use async_trait::async_trait;


cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod xterm;
        mod bindings;
        pub use xterm::Xterm as Interface;


    } else {
        mod native;
        pub use native::Termion as Interface;
    }
}

pub struct Options{
    pub prompt: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            prompt: None
        }
    }
}

impl Options{
    pub fn new() -> Options{
        Options::default()
    }

    pub fn with_prompt(mut self, prompt: String) -> Self {
        self.prompt = Some(prompt);
        self
    }

    pub fn prompt(&self) -> String {
        self.prompt.as_ref().unwrap_or(&"$ ".to_string()).clone()
    }
}


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

    pub fn reset_line_buffer(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }
}

#[async_trait]
// pub trait Cli : Sync + Send {
pub trait Cli {
    fn init(&self, _term : &Arc<Terminal>) -> Result<()> { Ok(()) }
    async fn digest(&self, term : Arc<Terminal>, cmd: String) -> CliResult<()>;
    async fn complete(&self, term : Arc<Terminal>, cmd : String) -> CliResult<Vec<String>>;
}

// unsafe impl Send for UserInput { }
// unsafe impl Sync for UserInput { }

#[derive(Clone)]
pub struct UserInput {
    buffer : Arc<Mutex<String>>,
    enabled : Arc<AtomicBool>,
    secure :  Arc<AtomicBool>,
    sender : Sender<String>,
    receiver : Receiver<String>,
}

impl UserInput {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        UserInput {
            buffer: Arc::new(Mutex::new(String::new())),
            enabled: Arc::new(AtomicBool::new(false)),
            secure:  Arc::new(AtomicBool::new(false)),
            sender,
            receiver,
        }
    }

    pub fn open(&self, secure : bool) -> Result<()> {
        self.enabled.store(true, Ordering::SeqCst);
        self.secure.store(secure, Ordering::SeqCst);
        Ok(())
    }

    pub fn close(&self) -> Result<()> {
        let s = {
            let mut buffer = self.buffer.lock().unwrap();
            let s = buffer.clone();
            buffer.clear();
            s
        };

        self.enabled.store(false, Ordering::SeqCst);
        self.sender.try_send(s).unwrap();
        Ok(())
    }

    pub async fn capture(&self, secure: bool) -> Result<String> {
        log_trace!("capturing...");
        self.open(secure)?;
        log_trace!("receiving...");
        let string = self.receiver.recv().await?;
        log_trace!("received!...");
        Ok(string)
    }

    fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }

    fn is_secure(&self) -> bool {
        self.secure.load(Ordering::SeqCst)
    }

    fn inject(&self, key : Key) -> Result<()> {
        match key {
            Key::Char(ch)=>{
                log_trace!("char...");

                self.buffer.lock().unwrap().push(ch);
            },
            Key::Backspace => {
                log_trace!("backspace...");

                self.buffer.lock().unwrap().pop();
            }
            Key::Enter => {
                log_trace!("closing...");
                self.close()?;
            }
            _ => { }
        }
        Ok(())
    }
    
}


unsafe impl Send for Terminal { }
unsafe impl Sync for Terminal { }

#[derive(Clone)]
pub struct Terminal {
    pub inner : Arc<Mutex<Inner>>,
    pub running: Arc<AtomicBool>,
    pub prompt : Arc<Mutex<String>>,
    pub term : Arc<Interface>,
    pub handler : Arc<dyn Cli>,
    pub terminate : Arc<AtomicBool>,
    user_input : UserInput,
}

impl Terminal {

    pub fn try_new(
        handler : Arc<dyn Cli>,
        prompt : &str,
    ) -> Result<Self> {

        let term = Arc::new(Interface::try_new()?);

        let terminal = Self {
            inner : Arc::new(Mutex::new(Inner::new())),
            running : Arc::new(AtomicBool::new(false)),
            prompt : Arc::new(Mutex::new(prompt.to_string())),
            term,
            handler,
            terminate : Arc::new(AtomicBool::new(false)),
            user_input : UserInput::new(),
        };

        Ok(terminal)
    }

    pub async fn init(self : &Arc<Self>)->Result<()>{
        self.handler.init(self)?;
        self.term.init(self).await?;
        Ok(())
    }

    pub fn inner(&self) -> LockResult<MutexGuard<'_, Inner>> {
        self.inner.lock()
    }

    pub fn reset_line_buffer(&self) {
        self.inner().unwrap().reset_line_buffer();
    }

    pub fn get_prompt(&self) -> String {
        return self.prompt.lock().unwrap().clone();
    }

    pub fn prompt(&self) {
        let mut data = self.inner().unwrap();
        data.cursor = 0;
		data.buffer = Vec::new();
        self.term().write(format!("{}", self.get_prompt()));
	}

    pub fn write<S>(&self, s : S) where S : Into<String> {
        self.term().write(s.into());
    }

    pub fn writeln<S>(&self, s : S) where S : Into<String> {
        self.write(format!("{}\n\x1B[2K\r", s.into()));
    }

    pub fn todo_writeln<S>(&self, s : S) -> Result<()> where S : Into<String> {
        
        let str : String = s.into() + "\r\n";
        // str.push("\r\n".to_string());
        
		if self.is_running() {
            self.write(str);
		}else {
            self.write(format!("\x1B[2K\r{}", str));
            let data = self.inner()?;
			let p = format!("{}{}", self.get_prompt(), data.buffer.join(""));
			self.write(p);            
			let l = data.buffer.len() - data.cursor;
			for _ in 0..l{
				self.write("\x08".to_string());
            }
		}

        Ok(())
	}

    pub fn term(&self) -> Arc<Interface> {
        return Arc::clone(&self.term);
    }

    pub async fn run(&self) -> Result<()> {
        self.prompt();
        Ok(self.term().run().await?)
    }

    pub fn exit(&self) {
        self.terminate.store(true, Ordering::SeqCst);
    }

    pub async fn ask(&self, secure: bool, prompt : &str) -> Result<String> {
        self.reset_line_buffer();
        self.term().write(prompt.to_string());
        Ok(self.user_input.capture(secure).await?)
    }

    pub fn inject(&self, text : String) -> Result<()> {
        let mut data = self.inner()?;
        self.inject_impl(&mut data, text)?;
        Ok(())
    }

    fn inject_impl(&self, data : &mut Inner, text : String) -> Result<()> {
        let len = text.len();
        let mut vec = data.buffer.clone();
        //log_trace!("before: vec.len(): {}", vec.len());
        let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), text.chars().map(|a|a.to_string())).collect();
        //let len_new = vec.len();
        data.buffer = vec;
        //log_trace!("after: vec.len(): {}, data.cursor:{}, buffer:{}, buffer.len():{}", len_new,  data.cursor+len, data.buffer.join(""), data.buffer.len());
        self.trail(data.cursor, &data.buffer, true, false, len);
        data.cursor = data.cursor+len;
        Ok(())
    }


    pub async fn ingest(self : &Arc<Terminal>, key : Key, _term_key : String) -> Result<()> {

        if self.user_input.is_enabled() {
            self.user_input.inject(key)?;
            return Ok(())
        }

        match key {
            Key::Backspace => {
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                self.write("\x08".to_string());
                data.cursor = data.cursor - 1;
                let mut vec = data.buffer.clone();
                vec.splice(data.cursor..(data.cursor+1), []);
                data.buffer = vec;
                self.trail(data.cursor, &data.buffer, true, true, 0);
            },
            Key::ArrowUp =>{
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
                self.write(format!("\x1B[2K\r{}{}", self.get_prompt(), data.buffer.join("")));
                data.cursor = data.buffer.len();
                
            }
            Key::ArrowDown =>{
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
                
                self.write(format!("\x1B[2K\r{}{}", self.get_prompt(), data.buffer.join("")));
                data.cursor = data.buffer.len();
            }
            Key::ArrowLeft =>{
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                data.cursor = data.cursor-1;
                self.write(Left(1));
            }
            Key::ArrowRight =>{
                let mut data = self.inner()?;
                if data.cursor < data.buffer.len() {
                    data.cursor = data.cursor+1;
                    self.write(Right(1).to_string());
                }
            }
            Key::Enter => {
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
                    self.writeln("");
                    self.running.store(true, Ordering::SeqCst);

                    let this = self.clone();
                    workflow_core::task::spawn(async move {
                            this.digest(cmd).await.ok();
                            this.running.store(false, Ordering::SeqCst);
                    });

                }else{
                    self.writeln("");
                    self.prompt();
                }
            },
            Key::Alt(_c)=>{
                return Ok(());
            },
            Key::Ctrl(_c)=>{
                return Ok(());
            },
            Key::Char(ch)=>{
                // let mut data = self.inner()?;
                self.inject(ch.to_string())?;
            },
            _ => {
                return Ok(());
            }
        }

        return Ok(());
    }

    fn trail(&self, cursor:usize, buffer:&Vec<String>, rewind: bool, erase_last : bool, offset : usize) {
		let mut tail = buffer[cursor..].join("");
        if erase_last{
            tail = tail+" ";
        }
		self.write(&tail);
        if rewind{
            let mut l = tail.len();
            if offset > 0{
                l = l-offset;
            }
            for _ in 0..l{
                self.write("\x08"); // backspace
            }
        }
	}

    pub fn is_running(&self)->bool{
        self.running.load(Ordering::SeqCst)
    }

    pub async fn digest(self : &Arc<Terminal>, cmd : String) -> Result<()> {
        if let Err(err) = self.handler.digest(self.clone(), cmd).await {
            self.writeln(format!("\x1B[2K\r{}", err));
        }
        if self.terminate.load(Ordering::SeqCst) {
            self.term().exit();
        } else {
            self.prompt();
        }
        Ok(())
    }

}

/// Utility function to strip multiple whitespaces and return a Vec<String>
pub fn parse(s : &str) -> Vec<String> {
    let regex = Regex::new(r"\s+").unwrap();
    let s = regex.replace_all(s, " ");
    s.split(' ').map(|s|s.to_string()).collect::<Vec<String>>()
}
