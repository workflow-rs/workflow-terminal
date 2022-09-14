


// pub trait Intake {
//     fn intake(key : Key);
// }

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, LockResult, atomic::AtomicBool};
use workflow_log::*;
use crate::result::Result;
use crate::keys::Key;
use crate::cursor::*;
use crate::spawn;
use std::future::Future;
use std::pin::Pin;

pub type HandleResult = Pin<Box<dyn Future<Output = Result<()>>>>;



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

/*
pub struct TerminalEvent{
    pub key:Key,
    pub key_str:String
}
*/


pub trait TerminalTrait : Sync + Send {
    fn write(&self, s: String) -> Result<()>;
    fn input_handler(&self, h:Cli)-> Result<()>;
    fn start(&self)-> Result<()>;
    //fn input_handler(&self, h:Box<dyn FnMut(TerminalEvent)-> Result<()>>)-> Result<()>;
}

pub trait Handler{
    //fn handle(&self, cmd:String)->Result<()>;

    fn handle(self: Arc<Self>, cmd:String)->HandleResult;
}

struct DefaultHandler{

}

impl Handler for DefaultHandler{
    fn handle(self: Arc<Self>, _cmd:String)->HandleResult{
        Box::pin(async move{
            Ok(())
        })
    }
}


#[derive(Clone)]
pub struct Cli {
    inner : Arc<Mutex<Inner>>,
    running: Arc<AtomicBool>,
    // term : Arc<Mutex<Arc<dyn Terminal>>>,
    term : Arc<dyn TerminalTrait>,
    prompt : Arc<Mutex<String>>,
    handler: Arc<Mutex<Arc<dyn Handler>>>,
}

impl Cli {

    pub fn new(
        term : Arc<dyn TerminalTrait>,
        prompt : Arc<Mutex<String>>,
    ) -> Result<Cli> {

        let cli = Cli {
            inner : Arc::new(Mutex::new(Inner::new())),
            running : Arc::new(AtomicBool::new(false)),
            term,
            prompt,
            handler: Arc::new(Mutex::new(Arc::new(DefaultHandler{}))),
        };

        cli.init()?;

        Ok(cli)
    }

    
    pub fn set_handler(&self, h: Arc<dyn Handler>)->Result<()>{
        let mut locked = self.handler.lock().expect("Unable to lock handler");
        *locked = h;
        Ok(())
    }

    fn init(&self)->Result<()>{
        let this = self.clone();
        /*
        self.term.input_handler(Box::new(move |e:TerminalEvent|->Result<()>{
            this.intake(e.key, e.key_str);
            Ok(())
        }))?;
        */
        self.term.input_handler(this)?;


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
        match key {
            Key::Backspace => {
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
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                data.cursor = data.cursor-1;
                self._write(Left(1))?;
            }
            Key::ArrowRight =>{
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
                //log_trace!("Key::Enter:cli");
                let cmd = {
                    let mut data = self.inner()?;
                    //e.stopPropagation();
                    //let { buffer, history } = this;
                    //let { length } = history;
                    let buffer = data.buffer.clone();
                    let length = data.history.len();

                    //self._write("\r\n")?;
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
                    self.running.store(true, Ordering::SeqCst);
                    self.digest(cmd)?;
                    self.running.store(false, Ordering::SeqCst);
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

    fn digest(&self, cmd: String) -> Result<()> {
        let this = self.clone();
        spawn(async move {
            log_trace!("Digesting: {}", cmd);
            let locked = this.handler.lock().expect("Unable to unlock handler");
            let _r = locked.clone().handle(cmd).await;
        });
        Ok(())
    }


}
