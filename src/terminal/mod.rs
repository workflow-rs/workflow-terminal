// extern crate termion;
use cfg_if::cfg_if;
use regex::Regex;
// use termion::event::Key as K;
// use termion::input::TermRead;
// use termion::raw::IntoRawMode;
// use termion::raw::RawTerminal;
// use std::io::{Write, Stdout, Stdin, stdout, stdin};
// //use workflow_log::*;
// use crate::cli::{Intake, Cli};//, Terminal as TerminalTrait};
// use crate::keys::Key;
// use crate::Result;
// use crate::Options;
// use std::sync::{Arc,Mutex};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, LockResult, atomic::AtomicBool};
//use workflow_log::*;
use crate::result::Result;
use crate::keys::Key;
use crate::cursor::*;
// use workflow_log::*;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod xterm;
        // pub use xterm::{Terminal, Options};
        pub use xterm::Xterm as Interface;


    } else {
        mod native;
        // pub use native::Options;
        pub use native::Termion as Interface;

        // ^ TODO load terminal
        
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


/* 
pub trait Term : Sync + Send {
    fn write(&self, s: String) -> Result<()>;
    fn start(&self)-> Result<()>;
    fn digest(&self, cmd: String) -> Result<()>;
    fn register_handler(&self, hander: Arc<dyn Cli>)-> Result<()>;
}


pub struct Terminal {
    intake: Option<Arc<Intake>>,
    handler: Option<Arc<dyn Cli>>,
    prompt : Option<Arc<Mutex<String>>>,
    term : Arc<Mutex<Option<Arc<Interface>>>>,
}

impl Terminal {
    pub fn new(cli : &Arc<dyn Cli>) -> Terminal {
        Self::new_with_options(cli, Options::default())
    }
    
    pub fn new_with_options(cli : &Arc<dyn Cli>, options : Options) -> Terminal {

        let term = Interface::new();

        Terminal {
            intake: Some(Arc::new(Intake::new(cli.clone()))),
            handler: Some(cli.clone()),
            term: None,
            prompt : None,
        }
    }

    // pub fn with_cli(mut self, cli : &Arc<dyn Cli>) -> Self {
    //     self.handler = Some(cli.clone());
    //     self.intake = Some(Arc::new(Intake::new(cli.clone())));
    //     self
    // }

    pub async fn init(self : &Arc<Self>) -> Result<()> {
        let mut term = Interface::new()
            .with_intake(&self.intake)
            .with_terminal(self);
        *self.term.lock()? = Some(Arc::new(term));
        Ok(())
    }

    pub async fn run(self) -> Result<()> {
        self.term.run().await?;
        Ok(())
    }

    pub fn term(&self) -> Arc<Interface> {
        self.term.lock()?.as_ref().unwrap()
    }

    pub fn intake(&self) -> Arc<Intake> {
        self.intake.clone()
    }

    pub fn handler(&self) -> Arc<dyn Cli> {
        self.handler.clone()
    }

    // ~~~

    pub fn write(&self, msg: &str) -> Result<()> {
        self.term().write(msg.to_string());
        Ok(())
    }

    pub fn writeln(&self, msg: &str) -> Result<()> {
        self.term().write(format!("{}\n\r",msg));
        Ok(())
    }


}

*/



use async_trait::async_trait;
//use workflow_log::log_trace;


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


// pub trait TerminalInterface : Sync + Send {
//     fn write(&self, s: String) -> Result<()>;
//     fn start(&self)-> Result<()>;
//     fn digest(&self, cmd: String) -> Result<()>;
//     fn register_handler(&self, hander: Arc<dyn Cli>)-> Result<()>;
// }

#[async_trait]
pub trait Cli : Sync + Send {
    fn init(&self, _term : &Arc<Terminal>) -> Result<()> { Ok(()) }
    async fn digest(&self, term : &Arc<Terminal>, cmd: String) -> Result<()>;
    async fn complete(&self, term : &Arc<Terminal>, substring : String) -> Result<Vec<String>>;
}

// pub struct DefaultHandler{}

// impl DefaultHandler{
//     pub fn new()->Self{
//         Self{}
//     }
// }

// #[async_trait]
// impl Cli for DefaultHandler{
//     async fn digest(&self, _cmd:String)->Result<()>{
//         Ok(())
//     }

//     async fn complete(&self, substring : String) -> Result<Vec<String>> {
//         if substring.starts_with('a') {
//             Ok(vec!["alpha".to_string(), "aloha".to_string(), "albatross".to_string()])
//         } else {
//             Ok(vec![])
//         }
//     }
// }

// pub struct ProcessResult{
//     pub texts: Vec<String>,
//     pub cmd: Option<String>
// }

// impl ProcessResult{
//     fn empty()->Self{
//         Self{texts:Vec::new(), cmd:None}
//     }
//     fn new(texts: Vec<String>)->Self{
//         Self{texts, cmd:None}
//     }
//     fn new_with_cmd(texts: Vec<String>, cmd:String)->Self{
//         Self{texts, cmd:Some(cmd)}
//     }
// }


#[derive(Clone)]
pub struct Terminal {
    pub inner : Arc<Mutex<Inner>>,
    pub running: Arc<AtomicBool>,
    pub prompt : Arc<Mutex<String>>,
    pub term : Arc<Interface>,
    pub handler : Arc<dyn Cli>,
    pub terminate : Arc<AtomicBool>,
}

impl Terminal {

    pub fn try_new(
        handler : Arc<dyn Cli>,
        prompt : &str, //Arc<Mutex<String>>,
    ) -> Result<Self> {

        let term = Arc::new(Interface::try_new()?);

        let terminal = Self {
            inner : Arc::new(Mutex::new(Inner::new())),
            running : Arc::new(AtomicBool::new(false)),
            prompt : Arc::new(Mutex::new(prompt.to_string())),
            term,
            handler,
            terminate : Arc::new(AtomicBool::new(false)),
        };

        // intake.init()?;

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

    pub fn get_prompt(&self) -> String {
        return self.prompt.lock().unwrap().clone();
    }

    pub fn prompt(&self) {
        /*
        let mut data = self.inner()?;
		self._prompt(&mut data)?;
        */
        let mut data = self.inner().unwrap();
        data.cursor = 0;
		data.buffer = Vec::new();

        // log_trace!("prompt...");

        // self.term().write(format!("\r\n{}", self.get_prompt()));
        self.term().write(format!("{}", self.get_prompt()));
		// Ok(format!("\r\n{}", self.prompt_str()))
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
    // pub fn write(&self, )



    // fn inject(&self, term_key : String) -> Result<String> {
    //     let mut data = self.inner()?;
    //     let mut vec = data.buffer.clone();
    //     let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), [term_key]).collect();
    //     data.buffer = vec;
    //     //log_trace!("inject: data.buffer: {:#?}", data.buffer);
    //     //log_trace!("inject: removed: {:#?}", removed);
    //     let texts = self.trail(data.cursor, &data.buffer, true, false, 1)?;

    //     data.cursor = data.cursor+1;
    //     Ok(texts)
    // }

    fn inject(&self, data : &mut Inner, term_key : String) -> Result<()> {
        // let mut data = self.inner()?;
        let mut vec = data.buffer.clone();
        // log_trace!("inject: vec: {}", vec.join(""));
        let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), [term_key]).collect();
        data.buffer = vec;
        //log_trace!("inject: data.buffer: {:#?}", data.buffer);
        //log_trace!("inject: removed: {:#?}", removed);
        self.trail(data.cursor, &data.buffer, true, false, 1);
        data.cursor = data.cursor+1;
        Ok(())
    }


    pub async fn ingest(self : &Arc<Terminal>, key : Key, _term_key : String) -> Result<()> {
        // let running = self.running.load(Ordering::SeqCst);
        let mut texts:Vec<String> = Vec::new();
        // fn empty()->Result<ProcessResult>{
        //     Ok(ProcessResult::empty())
        // }
        match key {
            Key::Backspace => {
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                texts.push("\x08".to_string());
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
            // "Inject"=>{
            //     inject(term_key);
            // }
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
                    // texts.push("\r\n".to_string());
                    // println!("starting digest...");
                    self.writeln("");
                    self.running.store(true, Ordering::SeqCst);
                    // return Ok(ProcessResult::new_with_cmd(texts, cmd));
                    // let result = 
                    self.digest(cmd).await.ok();
        
                    self.running.store(false, Ordering::SeqCst);
                    // println!("calling prompt...");
                    // self.prompt();
                    //#[cfg(not(target_arch="wasm32"))]
                    //self.after_digest()?;
                }else{
                    self.writeln("");
                    self.prompt();
                    // texts.push(self.prompt()?);

                }
            },
            Key::Alt(_c)=>{
                return Ok(());
            },
            Key::Ctrl(_c)=>{
                return Ok(());
            },
            Key::Char(ch)=>{
                let mut data = self.inner()?;
                self.inject(&mut data, ch.to_string())?;
            },
            _ => {
                return Ok(());
            }
        }

        return Ok(());
    }

    // pub fn after_digest(&self)-> Result<String> {
    //     self.running.store(false, Ordering::SeqCst);
    //     let text = self.prompt()?;
    //     Ok(text)
    // }

    // fn trail(&self, cursor:usize, buffer:&Vec<String>, rewind: bool, erase_last : bool, offset : usize) ->Result<String>{
	// 	let mut texts = Vec::new();
    //     let mut tail = buffer[cursor..].join("");
    //     if erase_last{
    //         tail = tail+" ";
    //     }
	// 	texts.push(tail.clone());
    //     if rewind{
    //         let mut l = tail.len();
    //         if offset > 0{
    //             l = l-offset;
    //         }
    //         for _ in 0..l{
    //             texts.push("\x08".to_string());//backspace
    //         }
    //     }
    //     Ok(texts.join(""))
	// }

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
                self.write("\x08");//backspace
            }
        }
	}



    pub fn is_running(&self)->bool{
        self.running.load(Ordering::SeqCst)
    }

    pub async fn digest(self : &Arc<Terminal>, cmd : String) -> Result<()> {
        // println!("digest 123");q
        if let Err(err) = self.handler.digest(self, cmd).await {
            self.writeln(format!("\x1B[2K\r{}", err));
        }
        // self.writeln("");
        if self.terminate.load(Ordering::SeqCst) {
            self.term().exit();
        } else {
            self.prompt();
        }
        Ok(())
    }

}


pub fn parse(s : &str) -> Vec<String> {
    let regex = Regex::new(r"\s+").unwrap();
    let s = regex.replace_all(s, " ");
    s.split(' ').map(|s|s.to_string()).collect::<Vec<String>>()
}
