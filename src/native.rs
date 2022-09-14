// extern crate termion;

use termion::event::Key as K;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use std::io::{Write, Stdout, Stdin, stdout, stdin};
use workflow_log::*;
use crate::cli::{Intake, TerminalTrait};
use crate::Cli;
use crate::keys::Key;
use crate::Result;
use std::sync::{Arc, Mutex};
use async_std::task;
//use async_std::sync::Mutex;


pub struct Terminal {
    cli: Arc<Mutex<Option<Cli>>>,
    intake: Arc<Mutex<Option<Intake>>>,
    //stdout:RawTerminal<Stdout>,
    //stdin:Stdin
}

impl Terminal {
    pub fn new() -> Result<Arc<Terminal>> {
        let stdout = stdout().into_raw_mode().unwrap();
        //let stdin = stdin();
        let mut terminal = Terminal {
            cli: Arc::new(Mutex::new(None)),
            intake: Arc::new(Mutex::new(None))
        };

        let term = terminal.init()?;
        *(term.intake.lock().await) =  Some(Intake::new(term.clone(), "$ ".to_string())?);
        Ok(term)
    }

    fn _write<S>(&self, s:S)->Result<()> where S:Into<String>{
        print!("{}", s.into());
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
        Ok(())
    }

    pub fn init(self)->Result<Arc<Self>> {
        let this = Arc::new(self);

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
        for c in stdin.keys() {
            /*
            write!(self.stdout,
                    "{}{}",
                    termion::cursor::Goto(1, 1),
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

            //print!("A");
            task::spawn(async move{
                let mut locked = self.intake.lock().expect("Unable to lock terminal.cli for intake");
                if let Some(intake) = locked.as_mut(){
                    //log_trace!("cli.intake: {:?}", key);
                    let _r = intake.intake(key, "".to_string()).await;
                }
            });


            //self.stdout.flush().unwrap();
        }

        Ok(())
    }
}

//impl Send for Terminal{}
//impl Sync for Terminal{}

impl TerminalTrait for Terminal{
    fn prompt(&self) -> Result<()>{
        let mut locked = self.intake.lock().expect("Unable to lock terminal.intake");
        if let Some(intake) = locked.as_mut(){
            intake.prompt()?;
        }
        Ok(())
    }

    fn write(&self, s: String) -> Result<()> {
        self._write(s)?;
        Ok(())
    }

    fn input_handler(&self, cli:Cli)-> Result<()> {
        let mut locked = self.cli.lock().expect("Unable to lock terminal.cli");
        *locked = Some(cli);
        Ok(())
    }

    fn start(&self)-> Result<()> {
        self._start()?;
        Ok(())
    }

    fn set_prompt(&self, prompt:String)-> Result<()>{
        let mut locked = self.intake.lock().expect("Unable to lock terminal.intake");
        if let Some(intake) = locked.as_mut(){
            intake.set_prompt(prompt)?;
        }
        
        Ok(())

    }
}