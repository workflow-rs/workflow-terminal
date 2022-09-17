use termion::event::Key as K;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use std::io::{Write, stdout, stdin};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::terminal::Terminal;
use crate::terminal::Options;
use crate::keys::Key;
use crate::Result;
use std::sync::{Arc,Mutex};

/// 
/// # Termion
/// 
/// Wrapper around Termion interface - https://crates.io/crates/termion
/// 
pub struct Termion {
    terminal: Arc<Mutex<Option<Arc<Terminal>>>>,
    terminate : AtomicBool,
}

impl Termion {
    pub fn try_new() -> Result<Self> {
        Self::try_new_with_options(Options::default())
    }
    pub fn try_new_with_options(_options:Options) -> Result<Self> {
        let termion = Termion {
            terminal: Arc::new(Mutex::new(None)),
            terminate : AtomicBool::new(false),
        };
        Ok(termion)
    }

    pub async fn init(self : &Arc<Self>, terminal : &Arc<Terminal>) -> Result<()> {
        *self.terminal.lock().unwrap() = Some(terminal.clone());
        Ok(())
    }

    pub fn exit(&self) {
        self.terminate.store(true, Ordering::SeqCst);
    }

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
                // keeping this for now (testing)
                K::Char('q') => break,
                K::Char(c) => {
                    if c == '\n' || c == '\r'{
                        Key::Enter
                    }else{
                        Key::Char(c)
                    }
                },
                K::Alt(c) => { Key::Alt(c) },
                K::Ctrl(c) => { Key::Ctrl(c) },
                K::Esc => { Key::Esc },
                K::Left => { Key::ArrowLeft },
                K::Right => { Key::ArrowRight },
                K::Up => { Key::ArrowUp },
                K::Down => { Key::ArrowDown },
                K::Backspace => { Key::Backspace },
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
        
            stdout.flush().unwrap();

            if self.terminate.load(Ordering::SeqCst) {
                break;
            }
        }

        Ok(())
    }

    pub fn write<S>(&self, s:S) where S:Into<String>{
        print!("{}", s.into());
            // stdout.flush().unwrap();
    }
    
}