use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::{cursor, event, execute,terminal};
use std::io::{stdout, Write};
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

struct Display 
{
    bordersize: (usize,usize)
}
impl Display {
    /* Association methods */
    fn create() -> Self { //creates terminmal window
        let bordersize = terminal::size()
        .map(|(x,y)|(x as usize, y as usize))
        .unwrap();
    
        Self { bordersize }
    }

    fn wipe_screen() -> crossterm::Result<()> //wipes screen, return type result so can either be successful or an error
    {
        execute!(stdout(), terminal::Clear(ClearType::All))?; // clear screen using execute macro, where the stdout specifies the output for the clear entire screen
        execute!(stdout(), cursor::MoveTo(0,0)) // reposition cursor to top left
    }

    fn borders(&self) {
        let line_start = self.bordersize.1; //prints <> on each line, similar to vim
        for x in 0..line_start {
            print!("<>");
        if x < line_start-1 {
            println!("\r");
        }
        stdout().flush();
    }
}
    
    
    fn refresh_screen(&self) -> crossterm::Result<()> // refresh screen 
    {
        Self::wipe_screen()?; // calls the type self (Display struct) to clear the screen, hence refreshing it
        self.borders();
        execute!(stdout(), cursor::MoveTo(0,0))
    }
}

struct EditKey
{ edit: DetectKey,
  display : Display,}

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
    fn start(&self) -> crossterm::Result<bool> //process events from DetectKey
    {
        self.display.refresh_screen()?;
        self.constructpress()

    }
}



fn main() -> crossterm::Result<()>
{

    let _broom = Sweep;
    terminal::enable_raw_mode()?;

    let system = EditKey::create();
    while system.start()? {}

    Ok(())


}
