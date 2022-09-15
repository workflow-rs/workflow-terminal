// extern crate termion;

use termion::event::Key as K;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use std::io::{Write, Stdout, Stdin, stdout, stdin};
//use workflow_log::*;
use crate::cli::{DefaultHandler, CliHandler, Terminal as TerminalTrait};
use crate::Cli;
use crate::keys::Key;
use crate::Result;
use std::sync::{Arc,Mutex};


pub struct Terminal {
    cli: Arc<Mutex<Option<Cli>>>,
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
            cli: Arc::new(Mutex::new(None)),
            handler: Arc::new(Mutex::new(Arc::new(DefaultHandler::new()))),
            //stdout,
            //stdin
        };

        let term = terminal.init()?;
        {
            let t = term.clone();
            let mut locked = t.cli.lock().expect("Unable to lock cli for init");
            *locked = Some(Cli::new(term.clone(), Arc::new(Mutex::new(opt.prompt)))?);
        }
        Ok(term)
    }

    fn _write<S>(&self, s:S)->Result<()> where S:Into<String>{
        println!("{}", s.into());
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
        Ok(())
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
            let mut locked = self.cli.lock().expect("Unable to lock terminal.cli for intake");
            if let Some(cli) = locked.as_mut(){
                //log_trace!("cli.intake");
                cli.intake(key, "".to_string())?;
            }


            //self.stdout.flush().unwrap();
        }

        Ok(())
    }

    pub fn write_str<S>(&self, text:S)->Result<()> where S:Into<String>{
        self._write(text.into())?;
        Ok(())
    }

    pub fn prompt(&self)->Result<()>{
        let locked = self.cli.lock().expect("Unable to lock cli for prompt");
        if let Some(cli) = locked.as_ref(){
            cli.prompt()?;
        }
        Ok(())
    }
}

//impl Send for Terminal{}
//impl Sync for Terminal{}


impl TerminalTrait for Terminal{
    fn write(&self, s: String) -> Result<()> {
        self._write(s)?;
        Ok(())
    }

    fn start(&self)-> Result<()> {
        self._start()?;
        Ok(())
    }
    fn digest(&self, cmd: String) -> Result<()>{
        println!("native-digest:cmd:{}", cmd);
        let this = self.clone();
        //let handler = self.handler.clone();
        //let cli = self.cli.clone();
        async_std::task::block_on(async move{
            println!("native-digest: AAA ");
            {
                let locked = this.handler.lock().expect("Unable to lock terminal.handler for digest");
                let _r = locked.digest(cmd).await;
            }
            println!("native-digest: BBB ");
            let mut locked_cli = this.cli.lock().expect("Unable to lock terminal.cli for digest");
            {
                println!("native-digest: CCCC ");
                if let Some(cli) = locked_cli.as_mut(){
                    println!("native-digest: DDD ");
                    let _r = cli.after_digest();
                }
            }
            println!("native-digest: EEEE ");
        });
        Ok(())
    }

    fn register_handler(&self, hander: Arc<dyn CliHandler>)-> Result<()> {
        let mut locked = self.handler.lock().expect("Unable to lock terminal.handler");
        *locked = hander;
        Ok(())
    }
}