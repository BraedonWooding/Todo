use termion::{clear, color, cursor};

use std::{time, thread};
use std::io::{stdin};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use termion::input::TermRead;

const COMMUNISM: &'static str = r#"
              !#########       #
            !########!          ##!
         !########!               ###
      !##########                  ####
    ######### #####                ######
     !###!      !####!              ######
       !           #####            ######!
                     !####!         #######
                        #####       #######
                          !####!   #######!
                             ####!########
          ##                   ##########
        ,######!          !#############
      ,#### ########################!####!
    ,####'     ##################!'    #####
  ,####'            #######              !####!
 ####'                                      #####
 ~##                                          ##~
"#;

pub fn begin_communism() {
    let original = Arc::new(AtomicBool::new(false));
    let stop = original.clone();

    let thread = thread::spawn(move || {
        let mut state = 0;
        println!("\n{}{}{}{}{}{}", cursor::Hide, clear::All, cursor::Goto(1, 1), color::Fg(color::Black), color::Fg(color::Red), COMMUNISM);
        loop {
            println!("{}{}           ☭ GAY ☭ SPACE ☭ COMMUNISM ☭           ", cursor::Goto(1, 1), color::Fg(color::AnsiValue(state)));
            println!("{}{}             WILL PREVAIL, COMRADES!             ", cursor::Goto(1, 20), color::Fg(color::AnsiValue(state)));

            state += 1;
            state %= 8;

            if stop.as_ref().load(Ordering::Relaxed) {
                break;
            }

            thread::sleep(time::Duration::from_millis(90));
        }
    });
    let stdin = stdin();

    for c in stdin.lock().keys() {
        match c {
            Ok(_) => break,
            _ => {},
        }
    }

    original.as_ref().store(true, Ordering::Relaxed);
    thread.join().unwrap();
}