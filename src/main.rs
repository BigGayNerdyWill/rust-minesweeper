use std::io;
use std::io::Write;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use termion::cursor::DetectCursorPos;
use rand::Rng;
use termion::color;

const MINEES:u16 = 99;
const HEIGHT:usize = 16;
const LENGTH:usize = 30;
const IBORDR:u16 = 1;
const JBORDR:u16 = 2;

fn genBoard(spawn:(usize,usize),brd:&mut [[u8;LENGTH];HEIGHT]){
    let (sx, sy) = spawn;
    let (sx,sy) = (sx as isize,sy as isize);
    let mut rng = rand::thread_rng();
    let mut count = 0;
    while count < MINEES {
        let (i,j) = (rng.gen_range(0..HEIGHT),rng.gen_range(0..LENGTH));
        let mut notspawn = false;
        let (di,dj) = ((i as isize -sx), (j as isize - sy));
        notspawn = (di > 1 || di < -1);
        notspawn |= (dj >1 || dj < -1);
        if brd[i][j] != 9 && notspawn{
            let mut ivals:Vec<usize> = vec![0,1,2];
            let mut jvals:Vec<usize> = vec![0,1,2];
            if i==0{ivals.remove(0);}
            if j==0{jvals.remove(0);}
            if j==LENGTH-1{jvals.remove(2);}
            if i==HEIGHT-1{ivals.remove(2);}
            for ii in ivals.iter(){
                for jj in jvals.iter(){
                    if brd[ii+i-1][jj+j-1] !=9{
                        brd[ii+i-1][jj+j-1]+=1;
                    }
                }
            }
            brd[i][j] = 9;
            count+=1;
        }
    }
}

fn revealTile(board:&mut [[u8;LENGTH];HEIGHT], known:&mut [[bool;LENGTH];HEIGHT],i:usize,j:usize){
    known[i][j] = true;
    if board[i][j]!=0{return;}
    let mut ivals:Vec<usize> = vec![0,1,2];
    let mut jvals:Vec<usize> = vec![0,1,2];
    if i==0{ivals.remove(0);}
    if j==0{jvals.remove(0);}
    if j==LENGTH-1{jvals.remove(2);}
    if i==HEIGHT-1{ivals.remove(2);}
    for ii in ivals.iter(){
        for jj in jvals.iter(){
            if (board[i+ii-1][j+jj-1]<9 && !known[i+ii-1][j+jj-1]){revealTile(board,known,i+ii-1,j+jj-1);}
        }
    }
}

fn draw(flgs:&mut [[bool;LENGTH];HEIGHT],known:&mut [[bool;LENGTH];HEIGHT],board:&mut [[u8;LENGTH];HEIGHT],coords:(usize,usize),lost:bool){
    let mut s = String::new();
    let mut stdout = io::stdout().into_raw_mode().unwrap();
    
    s.push_str("██");
    for j in 0..LENGTH{s.push('█');}
    s.push_str("██\n\r██");
    for i in 0..HEIGHT{
        for j in 0..LENGTH{
            if flgs[i][j] && !lost{
                s.push_str(format!("{}F{}",color::Fg(color::AnsiValue::rgb(5,0,0)),color::Fg(color::Reset)).as_str());
            }else if board[i][j]==9 && lost{
                s.push_str(format!("{}M{}",color::Fg(color::AnsiValue::rgb(5,0,0)),color::Fg(color::Reset)).as_str());
            }else if known[i][j] || lost{
                if board[i][j]==0 {
                    s.push('/');
                }else{
                    let mut c = board[i][j];
                    if (c>5){c=5;}
                    s.push_str(format!("{}{}{}",color::Fg(color::AnsiValue::rgb(c,5-c,0)),(board[i][j]+48) as char,color::Fg(color::Reset)).as_str());
                }
            }else{s.push(' ');}
        }
        s.push_str("██\n\r██");
    }
    for j in 0..LENGTH{s.push('█');}
    s.push_str("██\n\r");
    let (i,j) = coords;


    write!(stdout,"{}{}{}{}",termion::clear::All,termion::cursor::Goto(1,1),s,termion::cursor::Goto((j+1) as u16+JBORDR,(i+1) as u16 + IBORDR)).unwrap();
    stdout.flush().unwrap();
}

fn loose(flags:&mut [[bool;LENGTH];HEIGHT],known:&mut [[bool;LENGTH];HEIGHT],board:&mut [[u8;LENGTH];HEIGHT])->bool{ 
    draw(flags,known,board,(HEIGHT+3,0),true);
    println!("\ryou have lost.\n\rpress space to retry, and press q to quit\r");
    let stdin = io::stdin();
    for c in stdin.keys(){
        match c.unwrap(){
            Key::Char(' ') => {return true;},
            Key::Char('q') => {return false;},
            _ => continue,
        }
    }
    return false;
}

fn main() {


    let mut playing = true;
    while playing{
        let mut fTurn = true;
        let mut flgs = [[false;LENGTH];HEIGHT];
        let mut known = [[false;LENGTH];HEIGHT];
        let mut board = [[0;LENGTH];HEIGHT];
        let mut stdout = io::stdout().into_raw_mode().unwrap();
        draw(&mut flgs,&mut known,&mut board,(0,0),false);
        let stdin = io::stdin();
        for c in stdin.keys(){
            match c.unwrap(){
                Key::Char(' ') => { //opens tile up
                    let (j,i) = stdout.cursor_pos().unwrap();
                    let (i,j) = ((i-1-IBORDR) as usize,(j-1-JBORDR) as usize);
                    if fTurn{genBoard((i,j), &mut board);fTurn=false;}
                    if board[i][j]==9{
                        playing =  loose(&mut flgs,&mut known,&mut board); //if this returns true, restart game
                        break;}
                    else{revealTile(&mut board,&mut known,i,j);}
                    draw(&mut flgs,&mut known,&mut board,(i,j),false);
                },
                Key::Char('q') => {write!(stdout,"{}{}",termion::clear::All,termion::cursor::Goto(1,1));playing=false;break;}, //quits game
                Key::Char('f') => { //flag a bomb, allows for unflagging the same way
                    let (j,i) = stdout.cursor_pos().unwrap();
                    if (i > IBORDR && j> JBORDR){
                        let (i,j) = ((i-1-IBORDR) as usize,(j-1-JBORDR) as usize);
                        let mut c:char;
                        if (i < HEIGHT && j < LENGTH){
                            flgs[i][j] = !flgs[i][j];
                            if flgs[i][j]{c='F';
                            }else{c=' ';}
                            write!(stdout,"{}{}{}{}",color::Fg(color::AnsiValue::rgb(5,0,0)),c,color::Fg(color::Reset),termion::cursor::Left(1));
                        }
                    }
                },
                Key::Up => {write!(stdout,"{}",termion::cursor::Up(1));}
                Key::Right => {write!(stdout,"{}",termion::cursor::Right(1));}
                Key::Down => {write!(stdout,"{}",termion::cursor::Down(1));}
                Key::Left => {write!(stdout,"{}",termion::cursor::Left(1));}
                _ => continue,
            }
            stdout.flush().unwrap();
        }
    }
}
