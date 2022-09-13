


// pub trait Intake {
//     fn intake(key : Key);
// }

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, LockResult, atomic::AtomicBool};
use workflow_log::*;
use crate::result::Result;
use crate::keys::Key;
use crate::cursor::*;


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
    fn write(&mut self, s: String) -> Result<()>;
}


#[derive(Clone)]
pub struct Cli {
    inner : Arc<Mutex<Inner>>,
    running: Arc<AtomicBool>,
    // term : Arc<Mutex<Arc<dyn Terminal>>>,
    term : Arc<dyn Terminal>,
    prompt : Arc<Mutex<String>>,
}

impl Cli {

    pub fn new(
        term : Arc<dyn Terminal>,
        prompt : Arc<Mutex<String>>,
    ) -> Cli {
        Cli {
            inner : Arc::new(Mutex::new(Inner::new())),
            running : Arc::new(AtomicBool::new(false)),
            // term : Arc::new(Mutex::new(term)),
            term,
            prompt : prompt,
        }
    }

    pub fn inner(&self) -> LockResult<MutexGuard<'_, Inner>> {
        self.inner.lock()
    }

    pub fn write<S>(&self, s : S) where S : Into<String> {
        self.term.write(s.into());
    }

    // pub fn term(&self) -> LockResult<MutexGuard<'_, Arc<dyn Terminal>>> {
    //     self.term.lock()
    // }

    fn prompt_str(&self) -> String {
        return self.prompt.lock().unwrap().clone();
    }

    // fn prompt_str(&self) -> String{
	// 	format!("{} ", self.prompt_prefix)
	// }

    fn prompt(&self) -> Result<()> {
        let data = self.inner()?;
		data.cursor = 0;
		data.buffer = Vec::new();

		self.term.write(format!("\r\n{}", self.prompt_str()));
		//self.promptLength = ("\r\n"+prompt).length;

        Ok(())
	}



    fn inject(&self, term_key : String) -> Result<()> {
        let mut data = self.inner()?;
        let mut vec = data.buffer.clone();
        log_trace!("inject: vec: {}", vec.join(""));
        let _removed:Vec<String> = vec.splice(data.cursor..(data.cursor+0), [term_key]).collect();
        data.buffer = vec;
        //log_trace!("inject: data.buffer: {:#?}", data.buffer);
        //log_trace!("inject: removed: {:#?}", removed);
        self.trail(data.cursor, &data.buffer, true, false, 1);
        data.cursor = data.cursor+1;
        Ok(())
    }

    pub fn intake(&self, key : Key, term_key : String) -> Result<()> {
        match key {
            Key::Backspace => {
                let mut data = self.inner()?;
                if data.cursor == 0{
                    return Ok(());
                }
                self.write("\x08");
                data.cursor = data.cursor - 1;
                let mut vec = data.buffer.clone();
                vec.splice(data.cursor..(data.cursor+1), []);
                data.buffer = vec;
                self.trail(data.cursor, &data.buffer, true, true, 0);
            },
            Key::ArrowUp =>{
                let mut data = self.inner()?;
                //log_trace!("ArrowUp");
                if data.history_index == 0{
                    return Ok(());
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
                self.write(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")));
                data.cursor = data.buffer.len();
                
            }
            Key::ArrowDown =>{
                let mut data = self.inner()?;
                //log_trace!("ArrowDown");
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
                
                self.write(format!("\x1B[2K\r{}{}", self.prompt_str(), data.buffer.join("")));
                data.cursor = data.buffer.len();
            }
            Key::ArrowLeft =>{
                let mut data = self.inner()?;
                //log_trace!("ArrowLeft");
                if data.cursor == 0{
                    return Ok(());
                }
                data.cursor = data.cursor-1;
                // self.term.write(term_key);
                self.write(Left(1));
            }
            Key::ArrowRight =>{
                let mut data = self.inner()?;
                //log_trace!("ArrowRight");
                if data.cursor < data.buffer.len() {
                    data.cursor = data.cursor+1;
                    self.write(Right(1));
                }
            }
            // "Inject"=>{
            //     inject(term_key);
            // }
            Key::Enter => {
                let cmd = {
                    let mut data = self.inner()?;
                    //e.stopPropagation();
                    //let { buffer, history } = this;
                    //let { length } = history;
                    let buffer = data.buffer.clone();
                    let length = data.history.len();

                    self.write("\r\n");
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
                        Some(cmd)
                    } else {
                        None
                    }
                };

                if let Some(cmd) = cmd {
                // data.running = true;
                    self.running.store(true, Ordering::SeqCst);
                    self.digest(&cmd);
                    self.running.store(false, Ordering::SeqCst);
                }

                // self._prompt(&mut data);
            },
            Key::Char(ch)=>{
                // let mut data = self.inner()?;

                let printable = true; //TODO
                if printable {
                    self.inject(ch.to_string());
                    // self.inject(term_key);
                }
            },
            _ => {
                return Ok(());
            }
        }

        Ok(())
    }

    fn trail(&self, x:usize, buffer:&Vec<String>, rewind: bool, erase_last : bool, offset : usize) {
		let mut tail = buffer[x..].join("");
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

    fn digest(&self, cmd: &str) -> Result<()> {
        log_trace!("Digesting: {}", cmd);
        Ok(())
    }


}
