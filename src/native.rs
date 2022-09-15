// extern crate termion;

use termion::event::Key as K;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use std::io::{Write, Stdout, Stdin, stdout, stdin};
//use workflow_log::*;
use crate::cli::{Intake, DefaultHandler, CliHandler, Terminal as TerminalTrait};
use crate::keys::Key;
use crate::Result;
use std::sync::{Arc,Mutex};


pub struct Terminal {
    intake: Arc<Intake>,
    handler: Arc<Mutex<Arc<dyn CliHandler>>>,
    //stdout:RawTerminal<Stdout>,
    //stdin:Stdin
}

pub struct Options{
    pub prompt:String
}

impl Terminal {
    pub fn new(opt:Options) -> Result<Arc<Terminal>> {
        //let stdout = stdout().into_raw_mode().unwrap();
        //let stdin = stdin();
        let terminal = Terminal {
            intake: Arc::new(Intake::new(Arc::new(Mutex::new(opt.prompt)))?),
            handler: Arc::new(Mutex::new(Arc::new(DefaultHandler::new()))),
            //stdout,
            //stdin
        };
        let term = terminal.init()?;
        Ok(term)
    }

    pub fn init(self)->Result<Arc<Self>> {
        let this = Arc::new(self);

        //let stdout = stdout().into_raw_mode().unwrap();
        //let stdin = stdin();

        /*
        write!(self.stdout,
            "{}{}q to exit. Type stuff, use alt, and so on.{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::cursor::Hide)
            .unwrap();
        self.stdout.flush().unwrap();

        
        // TODO - FEED KEYSTROKE TO CLI
        write!(self.stdout, "{}", termion::cursor::Show).unwrap();

        write!(self.stdout, "sssssssss").unwrap();
        */

        Ok(this)
    }

    fn _start(&self)->Result<()> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();
        for c in stdin.keys() {
            
            /*
            write!(stdout,
                    "{}",
                    termion::clear::CurrentLine)
                    .unwrap();
            */
            

            //log_trace!("e:{:?}", c);
            let key = 
            match c.unwrap() {
                K::Char('q') => break,
                K::Char(c) => {//println!("{}", c);
                    if c == '\n' || c == '\r'{
                        //print!("enter: {}", c);
                        Key::Enter
                    }else{
                        Key::Char(c)
                    }
                },
                K::Alt(c) => {//println!("^{}", c)
                    Key::Alt(c)
                },
                K::Ctrl(c) =>{//println!("*{}", c)
                    Key::Ctrl(c)
                },
                K::Esc => {//println!("ESC")
                    Key::Esc
                },
                K::Left =>{//println!("←"),
                    Key::ArrowLeft
                },
                K::Right =>{//println!("→")
                    Key::ArrowRight
                },
                K::Up =>{//println!("↑")
                    Key::ArrowUp
                },
                K::Down =>{//println!("↓")
                    Key::ArrowDown
                },
                K::Backspace =>{//println!("×")
                    Key::Backspace
                },
                _ => {
                    continue;
                }
            };

            let res = self.intake.process_key(key, "".to_string())?;
        
            for text in res.texts{
                self.term_write(text);
            }
            if let Some(cmd) = res.cmd{
                self.digest(cmd)?;
            }

            stdout.flush().unwrap();
        }

        Ok(())
    }

    fn write_vec(&self, mut str_list:Vec<String>) ->Result<()> {
        let data = self.intake.inner()?;
		
        str_list.push("\r\n".to_string());
        
		if self.intake.is_running(){
			self.term_write(str_list.join(""));
		}else {
			self.term_write(format!("\x1B[2K\r{}", str_list.join("")));
			let prompt = format!("{}{}", self.intake.prompt_str(), data.buffer.join(""));
			self.term_write(prompt);
			let l = data.buffer.len() - data.cursor;
			for _ in 0..l{
				self.term_write("\x08".to_string());
            }
		}

        Ok(())
	}

    fn term_write<S>(&self, s:S) where S:Into<String>{
        print!("{}", s.into());
        //print!("{}", s.into());
        //let mut stdout = stdout().into_raw_mode().unwrap();
        /*write!(stdout,
            //"{}{}{}{}",
            "{}{}",
            //termion::clear::All,
            termion::cursor::Goto(1, 1),
            s.into(),
            //termion::cursor::Hide
            )
            .unwrap();*/
        //stdout.flush().unwrap();
    }

    pub fn write_str<S>(&self, text:S)->Result<()> where S:Into<String>{
        self.term_write(text.into());
        Ok(())
    }

    pub fn prompt(&self)->Result<()>{
        self.term_write(self.intake.prompt()?);
        Ok(())
    }
}

//impl Send for Terminal{}
//impl Sync for Terminal{}


impl TerminalTrait for Terminal{
    fn write(&self, s: String) -> Result<()> {
        self.write_vec(Vec::from([s]))?;
        Ok(())
    }

    fn start(&self)-> Result<()> {
        self._start()?;
        Ok(())
    }
    fn digest(&self, cmd: String) -> Result<()>{
        //println!("native-digest:cmd:{}", cmd);
        let this = self.clone();
        //let handler = self.handler.clone();
        //let cli = self.cli.clone();
        async_std::task::block_on(async move{
            //println!("native-digest: AAA ");
            
                let locked = this.handler.lock().expect("Unable to lock terminal.handler for digest");
                match locked.digest(cmd).await{
                    Ok(_)=>{
                        //let _r = this.term_write(text);
                    }
                    Err(e)=>{
                        let _r = this.term_write(e.to_string());
                    }
                }
            
            //println!("native-digest: BBB ");
            match this.intake.after_digest(){
                Ok(text)=>{
                    let _r = this.term_write(text);
                }
                Err(_e)=>{
                    //
                }
            }

            //println!("native-digest: EEEE ");
        });
        Ok(())
    }

    fn register_handler(&self, hander: Arc<dyn CliHandler>)-> Result<()> {
        let mut locked = self.handler.lock().expect("Unable to lock terminal.handler");
        *locked = hander;
        Ok(())
    }
}