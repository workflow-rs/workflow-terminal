

//use std::sync::atomic::Ordering;
use std::sync::{Arc};//, Mutex, MutexGuard, LockResult, atomic::AtomicBool};
use workflow_log::*;
use crate::result::Result;
use crate::keys::Key;
use crate::cursor::*;

/*

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
*/

/*
pub struct TerminalEvent{
    pub key:Key,
    pub key_str:String
}
*/


pub trait TerminalTrait : Sync + Send {
    fn write(&self, s: String) -> Result<()>;
    fn prompt(&self) -> Result<()>;
    fn input_handler(&self, h:Cli)-> Result<()>;
    fn start(&self)-> Result<()>;
    fn set_prompt(&self, prompt:String)-> Result<()>;
    //fn input_handler(&self, h:Box<dyn FnMut(TerminalEvent)-> Result<()>>)-> Result<()>;
}


pub struct Cli{
    term : Arc<dyn TerminalTrait>,
    
}

impl Cli {
    pub fn new(
        term : Arc<dyn TerminalTrait>,
        prompt: String
    ) -> Result<Self> {
        term.set_prompt(prompt)?;
        let cli = Self {
            term
        };

        cli.init()?;

        Ok(cli)
    }

    fn init(&self)->Result<()>{

        Ok(())
    }

    pub fn prompt(&self)->Result<()>{
        self.term.prompt()?;
        Ok(())
    }
}


pub struct Intake {
    term: Arc<dyn TerminalTrait>,
    running: bool,
    prompt_str : String,
    buffer:Vec<String>,
    history:Vec<Vec<String>>,
    cursor:usize,
    history_index:usize,
}

impl Intake {

    pub fn new(
        term: Arc<dyn TerminalTrait>,
        prompt : String,
    ) -> Result<Self> {
        let intake = Intake {
            term,
            running : false,
            prompt_str: prompt,
            buffer:Vec::new(),
            history:Vec::new(),
            cursor:0,
            history_index:0,
        };

        intake.init()?;

        Ok(intake)
    }

    fn init(&self)->Result<()>{
        //let this = self.clone();
        //self.term.input_handler(this)?;

        Ok(())
    }

    pub fn start(&self)->Result<()>{
        //self.term.start()?;
        Ok(())
    }

    pub fn set_prompt(&mut self, prompt:String)->Result<()>{
        self.prompt_str = prompt;
        Ok(())
    }

    fn write_vec(&self, mut str_list:Vec<String>) ->Result<()> {
		
        str_list.push("\r\n".to_string());
        
		if self.running {
			self.term_write(str_list.join(""))?;
		}else{
			self.term_write(format!("\x1B[2K\r{}{}{}", str_list.join(""), &self.prompt_str, self.buffer.join("")))?;
			
            let l = self.buffer.len() - self.cursor;
			for _ in 0..l{
				self.term_write("\x08".to_string())?;
            }
		}

        Ok(())
	}

    fn term_write<S>(&self, s : S)->Result<()> where S : Into<String> {
        self.term.write(s.into())?;
        Ok(())
    }

	pub fn write<S>(&self, s : S)-> Result<()>  where S : Into<String> {
        let s:String = s.into();
		self.write_vec(Vec::from([s]))?;
        Ok(())
	}

    pub fn prompt(&mut self) -> Result<()> {
        self.cursor = 0;
		self.buffer = Vec::new();

		self.term_write(format!("\r\n{}", &self.prompt_str))?;
        Ok(())
	}

    fn inject(&mut self, term_key : String) -> Result<()> {
        let mut vec = self.buffer.clone();
        let _removed:Vec<String> = vec.splice(self.cursor..(self.cursor+0), [term_key]).collect();
        self.buffer = vec;
        log_trace!("inject: self.buffer: {:#?}", self.buffer);
        //log_trace!("inject: removed: {:#?}", removed);
        self.trail(self.cursor, &self.buffer, true, false, 1)?;

        self.cursor = self.cursor+1;
        Ok(())
    }

    pub async fn intake(&mut self, key : Key, _term_key : String) -> Result<()> {
        match key {
            Key::Backspace => {
                if self.cursor == 0{
                    return Ok(());
                }
                self.term_write("\x08")?;
                self.cursor = self.cursor - 1;
                let mut vec = self.buffer.clone();
                vec.splice(self.cursor..(self.cursor+1), []);
                self.buffer = vec;
                self.trail(self.cursor, &self.buffer, true, true, 0)?;
            },
            Key::ArrowUp =>{
                if self.history_index == 0{
                    return Ok(());
                }
                let current_buffer = self.buffer.clone();
                let index = self.history_index;
                //log_trace!("ArrowUp: index {}, self.history.len(): {}", index, self.history.len());
                if self.history.len() <= index{
                    self.history.push(current_buffer);
                }else{
                    self.history[index] = current_buffer;
                }
                self.history_index = self.history_index-1;
                
                self.buffer = self.history[self.history_index].clone();
                self.term_write(format!("\x1B[2K\r{}{}", &self.prompt_str, self.buffer.join("")))?;
                self.cursor = self.buffer.len();
                
            }
            Key::ArrowDown =>{
                let len =  self.history.len();
                if self.history_index >= len{
                    return Ok(());
                }
                let index = self.history_index;
                self.history[index] = self.buffer.clone();
                self.history_index = self.history_index+1;
                if self.history_index == len{
                    self.buffer = Vec::new();
                }else{
                    self.buffer = self.history[self.history_index].clone();
                }
                
                self.term_write(format!("\x1B[2K\r{}{}", &self.prompt_str, self.buffer.join("")))?;
                self.cursor = self.buffer.len();
            }
            Key::ArrowLeft =>{
                if self.cursor == 0{
                    return Ok(());
                }
                self.cursor = self.cursor-1;
                self.term_write(Left(1))?;
            }
            Key::ArrowRight =>{
                if self.cursor < self.buffer.len() {
                    self.cursor = self.cursor+1;
                    self.term_write(Right(1))?;
                }
            }
            // "Inject"=>{
            //     inject(term_key);
            // }
            Key::Enter => {
                //log_trace!("Key::Enter:cli");
                let cmd = {
                    //e.stopPropagation();
                    //let { buffer, history } = this;
                    //let { length } = history;
                    let buffer = self.buffer.clone();
                    let length = self.history.len();

                    //self.term_write("\r\n")?;
                    self.buffer = Vec::new();
                    self.cursor = 0;

                    if buffer.len() > 0 {
                        
                        let cmd = buffer.join("");
                        if length==0 || self.history[length-1].len() > 0{
                            self.history_index = length;
                        }else{
                            self.history_index = length-1;
                        }
                        let index = self.history_index;
                        //log_trace!("length:{length},  history_index:{index}");
                        if length<=index {
                            self.history.push(buffer);
                        }else{
                            self.history[index] = buffer;
                        }
                        self.history_index = self.history_index+1;

                        //log_trace!("length222:{length},  history_index:{}, {}", self.history_index, cmd);
                        Some(cmd)
                    } else {
                        None
                    }
                };

                if let Some(cmd) = cmd {
                    self.running = true;
                    self.digest(&cmd).await?;
                    self.running = false;
                }

                self.prompt()?;
            },
            Key::Alt(_c)=>{
                return Ok(());
            },
            Key::Ctrl(_c)=>{
                return Ok(());
            },
            Key::Char(ch)=>{
                self.inject(ch.to_string())?;
            },
            _ => {
                return Ok(());
            }
        }

        Ok(())
    }

    fn trail(&self, cursor:usize, buffer:&Vec<String>, rewind: bool, erase_last : bool, offset : usize) ->Result<()>{
		let mut tail = buffer[cursor..].join("");
        if erase_last{
            tail = tail+" ";
        }
		self.term_write(&tail)?;
        if rewind{
            let mut l = tail.len();
            if offset > 0{
                l = l-offset;
            }
            for _ in 0..l{
                self.term_write("\x08")?;//backspace
            }
        }
        Ok(())
	}

    async fn digest(&self, cmd: &str) -> Result<()> {
        log_trace!("Digesting: {}", cmd);
        Ok(())
    }


}
