use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{cursor, event, execute,terminal, queue};
use std::io::{stdout, Write, self};
use std::time::Duration; 
use crossterm::terminal::ClearType; 
 

struct Sweep; //brings terminal out of raw mode and back to canonical mode 

impl Drop for Sweep { //this has been implemented as a struct as if the enable function panics, it may not exit raw mode
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Failed to disbale RAW");
        Display::wipe_screen().expect("Could not wipe text") // clear screen on exit
    }
}

struct DetectKey;
impl DetectKey {
    fn keydetection(&self) -> crossterm::Result<KeyEvent> //borrows the instance of the struct -> not mutable as i only want to read from the struct
    {
        loop { // loop over each key press
            if(event::poll(Duration::from_millis(1000)))? //sets a timelimimt for the next read function, if no event occurs, false is returned
            {
                if let Event::Key(event) = event::read()? // checks if the event "keypress" is actually a key
                {
                    return Ok(event); // return an enum, in this case if its a key, its accepted
                }
            }
        }
    }
}

struct AppendBuffer
{
    content: String
}

impl AppendBuffer
{
    fn new() -> Self
    {
        Self { content: String::new() } //constructor for instance of appendbuffer
    }

    fn push(&mut self, ch: char) //append a character to the buffer
    {
        self.content.push(ch)
    }

    fn push_string(&mut self, string: &str) //append a string to the buffer
    {
        self.content.push_str(string)
    }

}

//since appendbuffer is a struct of type AppendBuffer, the io::write must be written explicitly for this type, so it can use the io::Write trait
impl io::Write for AppendBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> { // 
        match std::str::from_utf8(buf) { // convert bytes into string so it can be added to "content", converts bytes if they're utf8 encoded
            Ok(s) => {
                self.content.push_str(s); //appends utf-8 string "s" to the content field for the buffer struct
                Ok(s.len()) //return the length of the stirng if bytes can be converted, 
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into()), // if cant be converted, return an error of write zero - zero bytes written
        }
    }


    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content); //write to the stdout using write! macro
        stdout().flush()?; //call flush on standard output stream -> ensures any buffered content is actually written out
        self.content.clear(); // clear content so it can be used for next screen refresh 
        out // end of write! macro, indicates whether the buffer content was written or not
    }
}
struct Display 
{
    bordersize: (usize,usize),
    content: AppendBuffer,
}
impl Display {
    /* Association methods */
    fn create() -> Self { //creates terminmal window
        let bordersize = terminal::size()
        .map(|(x,y)|(x as usize, y as usize))
        .unwrap();
    
        Self { bordersize,
        content: AppendBuffer::new(),
     } 
    }


    fn wipe_screen() -> crossterm::Result<()> //wipes screen, return type result so can either be successful or an error
    {
        execute!(stdout(), terminal::Clear(ClearType::All))?; // clear screen using execute macro, where the stdout specifies the output for the clear entire screen
        execute!(stdout(), cursor::MoveTo(0,0)) // reposition cursor to top left
    }

    fn borders(&mut self) {
        let line_start = self.bordersize.1; //prints <> on each line, similar to vim
        for x in 0..line_start {
            self.content.push('>');
            if x < line_start - 1 {
                self.content.push_string("\r\n");
            }
        }
    }
    
    
    fn refresh_screen(&mut self) -> crossterm::Result<()> // refresh screen 
    {
        
        queue!(self.content, terminal::Clear(ClearType::All), cursor::MoveTo(0,0)); //waits for flush to be called then it is executed
        self.borders();
        queue!(self.content, cursor::MoveTo(0,0))?; //wait for flush to be called 
        self.content.flush()
    }
}


struct EditKey
{ edit: DetectKey,
  display : Display,
  }

impl EditKey { //this is the "engine" of the whole system, so it will be called in main to produce the system

    fn create() -> Self { // create initialises the struct "EditKey" and returns an instance of itself 
        Self {edit: DetectKey,
        display: Display::create(), } // shorthand and initialises the edit field with a detectkey struct
    }
    
    fn constructpress(&self) -> crossterm::Result<bool> // function that regusters key inputs, returns false to quit the editor and true for every other key press
    {
            match self.edit.keydetection()? // looks for the sequence "ALT+Q" to quit the editor
            {
                KeyEvent //keyevent
                {
                    code: KeyCode::Char('q'), // sequence to exit the editor
                    modifiers: event::KeyModifiers::ALT,
                } => return Ok(false), //return false to quit the editor
                _ => {} //  catch all - do nothing for other key presses
            }
            return Ok(true) // true for all other key presses that != ALT+Q
    }
    fn start(&mut self) -> crossterm::Result<bool> //process events from DetectKey
    {
        self.display.refresh_screen()?;
        self.constructpress()

    }
}



fn main() -> crossterm::Result<()>
{

    let _broom = Sweep;
    terminal::enable_raw_mode()?;
    let mut spud = EditKey::create();
    while spud.start()? {}

    Ok(())


}
