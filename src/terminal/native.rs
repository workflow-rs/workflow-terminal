// extern crate termion;

use termion::event::Key as K;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
// use termion::raw::RawTerminal;
// use std::io::{Write, Stdout, Stdin, stdout, stdin};
use std::io::{Write, stdout, stdin};
use std::sync::atomic::{AtomicBool, Ordering};
//use workflow_log::*;
// use crate::cli::{Intake, Cli};
use crate::terminal::Terminal;
use crate::terminal::Options;
// use crate::terminal::Term;
use crate::keys::Key;
use crate::Result;
// use crate::terminal::Options;
use std::sync::{Arc,Mutex};




pub struct Termion {
    // prompt: Arc<Mutex<String>>,
    // intake: Option<Arc<Terminal>>,
    terminal: Arc<Mutex<Option<Arc<Terminal>>>>,
    terminate : AtomicBool,
    // handler: Option<Arc<dyn Cli>>,
    //stdout:RawTerminal<Stdout>,
    //stdin:Stdin
}

// pub struct Options{
//     pub prompt:String
// }

impl Termion {
    pub fn try_new() -> Result<Self> {
        Self::try_new_with_options(Options::default())
    }
    pub fn try_new_with_options(_options:Options) -> Result<Self> {
        //let stdout = stdout().into_raw_mode().unwrap();
        //let stdin = stdin();

        // let prompt = Arc::new(Mutex::new(options.prompt()));

        let termion = Termion {
            // prompt,
            // intake: Arc::new(Intake::new(Arc::new(Mutex::new(opt.prompt())))?),
            // intake: None, //Arc::new(Intake::new(prompt)),
            terminal: Arc::new(Mutex::new(None)),
            terminate : AtomicBool::new(false),
            // handler: Arc::new(Mutex::new(Arc::new(DefaultHandler::new()))),
            // handler: None, // Arc::new(DefaultHandler::new()),
            //stdout,
            //stdin
        };
        // let term = terminal.init()?;
        Ok(termion)
    }

    pub async fn init(self : &Arc<Self>, terminal : &Arc<Terminal>) -> Result<()> {
        *self.terminal.lock().unwrap() = Some(terminal.clone());
        Ok(())
    }

    pub fn exit(&self) {
        self.terminate.store(true, Ordering::SeqCst);
    }

    // pub fn with_intake(mut self, intake: &Arc<Intake>) -> Termion {
    //     self.intake = Some(intake.clone());
    //     self
    // }

    // pub fn terminal(&self) -> Arc<Terminal> {
    //     self.terminal.lock().unwrap().clone()
    // }

    // pub fn init(self)->Result<Arc<Self>> {
    //     let this = Arc::new(self);

    //     //let stdout = stdout().into_raw_mode().unwrap();
    //     //let stdin = stdin();

    //     /*
    //     write!(self.stdout,
    //         "{}{}q to exit. Type stuff, use alt, and so on.{}",
    //         termion::clear::All,
    //         termion::cursor::Goto(1, 1),
    //         termion::cursor::Hide)
    //         .unwrap();
    //     self.stdout.flush().unwrap();

        
    //     // TODO - FEED KEYSTROKE TO CLI
    //     write!(self.stdout, "{}", termion::cursor::Show).unwrap();

    //     write!(self.stdout, "sssssssss").unwrap();
    //     */

    //     Ok(this)
    // }

    pub async fn run(&self)->Result<()> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode().unwrap();
        stdout.flush().unwrap();

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

            let _result = self.terminal
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .ingest(key, "".to_string()).await;
        
            // if let Err(err) = result {
            //     self.writeln(format!("\x1B[2K\r{}", err));
            // }
            // for text in res.texts{
            //     self.term_write(text);
            // }
            // if let Some(cmd) = res.cmd{
            //     self.digest(cmd)?;
            // }

            stdout.flush().unwrap();

            if self.terminate.load(Ordering::SeqCst) {
                break;
            }
        }

        Ok(())
    }

    // fn write_vec(&self, mut str_list:Vec<String>) ->Result<()> {
    //     let terminal = self.terminal.lock().unwrap().as_ref().unwrap().clone();
    //     let data = terminal.inner()?;
		
    //     str_list.push("\r\n".to_string());
        
	// 	if intake.is_running(){
	// 		self.term_write(str_list.join(""));
	// 	}else {
	// 		self.term_write(format!("\x1B[2K\r{}", str_list.join("")));
	// 		let prompt = format!("{}{}", intake.prompt_str(), data.buffer.join(""));
	// 		self.term_write(prompt);
	// 		let l = data.buffer.len() - data.cursor;
	// 		for _ in 0..l{
	// 			self.term_write("\x08".to_string());
    //         }
	// 	}

    //     Ok(())
	// }

    pub fn write<S>(&self, s:S) where S:Into<String>{
        print!("{}", s.into());
            // stdout.flush().unwrap();
    }

    
    // pub fn writeln<S>(&self, s:S) where S:Into<String>{
    //     print!("{}\n\x1B[2K\r", s.into());
    //     // stdout.flush().unwrap();

    // }


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
    // }

    // pub fn write_str<S>(&self, text:S)->Result<()> where S:Into<String>{
    //     self.term_write(text.into());
    //     Ok(())
    // }

    // pub fn prompt(&self)->Result<()>{
    //     self.term_write(self.intake.as_ref().unwrap().prompt()?);
    //     Ok(())
    // }
// }

// //impl Send for Terminal{}
// //impl Sync for Terminal{}


// impl Term for Termion {
    // fn write(&self, s: String) -> Result<()> {
    //     self.write_vec(Vec::from([s]))?;
    //     Ok(())
    // }

    // fn start(&self)-> Result<()> {
    //     self._start()?;
    //     Ok(())
    // }
    // fn digest(&self, cmd: String) -> Result<()>{
    //     //println!("native-digest:cmd:{}", cmd);
    //     let this = self.clone();
    //     //let handler = self.handler.clone();
    //     //let cli = self.cli.clone();
    //     async_std::task::block_on(async move{
    //         //println!("native-digest: AAA ");
            
    //             let locked = this.handler.lock().expect("Unable to lock terminal.handler for digest");
    //             match locked.digest(cmd).await{
    //                 Ok(_)=>{
    //                     //let _r = this.term_write(text);
    //                 }
    //                 Err(e)=>{
    //                     let _r = this.term_write(e.to_string());
    //                 }
    //             }
            
    //         //println!("native-digest: BBB ");
    //         match this.intake.after_digest(){
    //             Ok(text)=>{
    //                 let _r = this.term_write(text);
    //             }
    //             Err(_e)=>{
    //                 //
    //             }
    //         }

    //         //println!("native-digest: EEEE ");
    //     });
    //     Ok(())
    // }

    // fn register_handler(&self, hander: Arc<dyn Cli>)-> Result<()> {
    //     let mut locked = self.handler.lock().expect("Unable to lock terminal.handler");
    //     *locked = hander;
    //     Ok(())
    // }
}