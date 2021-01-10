use std::io;
use std::io::Write;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use termion::cursor::DetectCursorPos;
use rand::Rng;
use termion::color;
use std::sync::mpsc;
use std::thread;

const MINEES:u16 = 99;
const HEIGHT:usize = 16;
const LENGTH:usize = 32;
const IBORDR:u16 = 1;
const JBORDR:u16 = 2;
const BOT:bool = true;

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
    if board[i][j]!=0 || BOT{return;}
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

fn gameOver(flags:&mut [[bool;LENGTH];HEIGHT],known:&mut [[bool;LENGTH];HEIGHT],board:&mut [[u8;LENGTH];HEIGHT],s:&str)->bool{ 
    draw(flags,known,board,(HEIGHT+3,0),true);
    //println!("\r{}\n\rpress space to retry, and press q to quit\r",s);
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


fn click(flgs:&mut [[bool;LENGTH];HEIGHT],known:&mut [[bool;LENGTH];HEIGHT],board:&mut [[u8;LENGTH];HEIGHT],fTurn:&mut bool, cords:(usize,usize)) -> bool{
    if *fTurn{genBoard(cords, board);*fTurn=false;}
    if board[cords.0][cords.1]==9{return true;}
    else {revealTile(board,known,cords.0,cords.1);}
    if !BOT {draw(flgs,known,board,cords,false);}
    false
}

fn placeFlag(flgs:&mut [[bool;LENGTH];HEIGHT],known:&mut [[bool;LENGTH];HEIGHT],board:&mut [[u8;LENGTH];HEIGHT], i:usize,j:usize,corr:&mut u16)->bool{ 
    let stdin = io::stdin();
    //if board[i][j]!=9{
    //    panic!("not a bomb");}
    flgs[i][j] = !flgs[i][j];
    known[i][j] = !known[i][j];
    if board[i][j]==9{if flgs[i][j]{*corr+=1;}else{*corr-=1;}}
    let mut c = ' ';
    if flgs[i][j]{c='F';}
    let mut s = String::new();
    if BOT{s = format!("{}",termion::cursor::Goto(j as u16 +JBORDR+1,i as u16 +IBORDR+1));}
    write!(io::stdout(),"{}{}{}{}{}",s,color::Fg(color::AnsiValue::rgb(5,0,0)),c,color::Fg(color::Reset),termion::cursor::Left(1));
    if *corr>=MINEES{return gameOver(flgs,known,board,"you have won!");}
    false
}

fn filledSpace(space:&mut [[u8;LENGTH];HEIGHT],i:usize,j:usize){
    let (ivals,jvals) = area(i,j);
    for ii in ivals.iter(){
        for jj in jvals.iter(){
            if (*ii!=1 || *jj!=1){space[ii+i-1][j+jj-1]-=1;}
        }
    }
}

fn bombFound(spaces:&mut [[u8;LENGTH];HEIGHT],board:&mut [[u8;LENGTH];HEIGHT],i:usize,j:usize){
    let (ivals,jvals) = area(i,j);
    let stdin = io::stdin();
    //if board[i][j]!=9{
    //    panic!("not a bomb");}
    for ii in ivals.iter(){
        for jj in jvals.iter(){
            if board[i+ii-1][j+jj-1]>0 && 9>board[i+ii-1][j+jj-1] && (1!=*jj || *ii!=1)
            {
                board[i+ii-1][j+jj-1]-=1;
                spaces[i+ii-1][j+jj-1]-=1;
            }
        }
    }
}
        
fn square5(i:usize,j:usize)->Vec<(usize,usize)>{
    let mut v = Vec::new();
    let mut ivals:Vec<usize> = vec![0,1,2,3,4];
    let mut jvals:Vec<usize> = vec![0,1,2,3,4];
    for k in 0..2{
        if j+k<=1{jvals.remove(0);}
        if i+k<=1{ivals.remove(0);}
        if j+2>=LENGTH+k{jvals.pop();}
        if i+2>=HEIGHT+k{ivals.pop();}
    }
    for ii in ivals.iter(){
        for jj in jvals.iter(){
            v.push((*ii,*jj));
        }
    }
    return v;
}

fn area(i:usize,j:usize)->(Vec<usize>,Vec<usize>){
    let mut ivals:Vec<usize> = vec![0,1,2];
    let mut jvals:Vec<usize> = vec![0,1,2];
    if i==0{ivals.remove(0);}
    if j==0{jvals.remove(0);}
    if j==LENGTH-1{jvals.remove(2);}
    if i==HEIGHT-1{ivals.remove(2);}
    return (ivals,jvals);
}

fn ptrn(currPat:Vec<(usize,usize)>,cont:usize,blocks:&mut Vec<(usize,usize)>) -> Vec<Vec<(usize,usize)>>{
    if currPat.len() >= cont{return vec![currPat];}
    let mut vout = Vec::new();
    let mut b2 = Vec::new();
    for x in blocks.iter(){b2.push(*x);}
    while b2.len()>0{
        let b = b2.pop().unwrap();
        let mut p = Vec::new();
        for x in currPat.iter(){
            p.push(*x);
        }
        p.push(b);
        let mut o = ptrn(p,cont,&mut b2);
        vout.append(&mut o);
    }
    return vout;
}

fn runBot(){
    let mut correct=0;
    let mut playing = true;
    let (tx,rx):(mpsc::Sender<bool>,mpsc::Receiver<bool>) = mpsc::channel();
    let t =thread::spawn(move ||{
        while playing{
            let mut failRow=0.0;
            correct=0;
            playing = false;
            let mut fTurn=true;
            let mut flgs = [[false;LENGTH];HEIGHT];
            let mut known = [[false;LENGTH];HEIGHT];
            let mut board = [[0;LENGTH];HEIGHT];
            let mut stdout = io::stdout().into_raw_mode().unwrap();
            draw(&mut flgs,&mut known,&mut board,(0,0),false);
            let mut spaces = [[8;LENGTH];HEIGHT];
            click(&mut flgs,&mut known,&mut board,&mut fTurn,(HEIGHT/2,LENGTH/2));
            filledSpace(&mut spaces,HEIGHT/2,LENGTH/2);
            let mut lchang=true;
            let mut chang=false;
            while !playing{
                for i in 0..HEIGHT{
                    for j in 0..LENGTH{
                        if known[i][j] && board[i][j]<9{
                            let (ivals,jvals) = area(i,j);
                            let mut tiles:Vec<(usize,usize)> = Vec::new();
                            for ii in ivals.iter(){
                                for jj in jvals.iter(){
                                    if((*ii == 1 && *jj == 1) || known[i+ii-1][j+jj-1]){continue;}
                                    if board[i][j]==0{
                                        chang=true;
                                        playing = click(&mut flgs,&mut known,&mut board,&mut fTurn,(i+ii-1,j+jj-1));
                                        filledSpace(&mut spaces,i+ii-1,j+jj-1);
                                    }else if board[i][j]==spaces[i][j]{
                                        chang=true;
                                        playing = placeFlag(&mut flgs,&mut known, &mut board,i+ii-1,j+jj-1,&mut correct);
                                        bombFound(&mut spaces,&mut board,i+ii-1,j+jj-1);
                                    }else{
                                        tiles.push((*ii,*jj));
                                    }
                                }
                            }
                            if !lchang && spaces[i][j]>0{
                                let mut wrong;
                                let mut outs = ptrn(Vec::new(),board[i][j] as usize,&mut tiles);
                                let m = square5(i,j);
                                let mut mb = [[0;5];5];
                                let mut corrPats:Vec<(usize,usize)>=Vec::new();
                                for o in outs.iter_mut(){
                                    //o is a vector of tiles, that should be tested to see if mines fit
                                    for t in m.iter(){
                                        mb[t.0][t.1] = board[t.0+i-2][t.1+j-2] as i8;
                                    }
                                    for posFlag in o.iter(){
                                       let (ivals,jvals) = area(i+posFlag.0-1,j+posFlag.1-1);
                                       for ii in ivals.iter(){
                                            for jj in jvals.iter(){
                                                mb[posFlag.0+ii][posFlag.1+jj]-=1;
                                            }
                                       }
                                    }
                                    wrong = false;
                                    for ii in 0..5{
                                        for jj in 0..5{
                                            if mb[ii][jj] < 0{
                                                wrong=true;
                                                break;
                                            }
                                        }
                                        if wrong{break;}
                                    }
                                    if !wrong{corrPats.append(o);}
                                }
                                let mut tprob:[[f32;3];3] = [[0.0;3];3];
                                let l = corrPats.len() as f32;
                                for pat in corrPats.iter(){
                                    tprob[pat.0][pat.1]+=1.0;
                                }
                                for ii in 0..3{
                                    for jj in 0..3{
                                        tprob[ii][jj] /= l;
                                        if tprob[ii][jj] >=1.0- failRow{
                                            chang=true;
                                            playing = placeFlag(&mut flgs,&mut known,&mut board,i+ii-1,j+jj-1,&mut correct);
                                            bombFound(&mut spaces,&mut board,i+ii-1,j+jj-1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !(chang||lchang){
                    failRow+=0.1;
                    if failRow>0.2{
                        //highest lowest p and then click it
                        //the others are gaurenteed to win, this isnt
                        //thread::sleep_ms(1000);
                        for i in 0..HEIGHT{
                            for j in 0..LENGTH{
                                if known[i][j]{
                                    let (ivec,jvec) = area(i,j);
                                    for ii in ivec.iter(){
                                        for jj in jvec.iter(){
                                            let p = (i+ii-1,j+jj-1); 
                                            if !known[p.0][p.1]{
                                                if board[p.0][p.1]==9{
                                                    chang=true;
                                                    playing = placeFlag(&mut flgs,&mut known, &mut board,p.0,p.1,&mut correct);
                                                    bombFound(&mut spaces,&mut board,p.0,p.1);
                                                }else{
                                                    chang=true;
                                                    playing = click(&mut flgs,&mut known,&mut board,&mut fTurn,p);
                                                    filledSpace(&mut spaces,p.0,p.1);
                                                }
                                            }
                                        }
                                    }
                                }
                                if chang{break;}
                            }
                            if chang{break;}
                        }
                        failRow=0.0;
                    }
                }
                else{failRow=0.0;}
                lchang = chang;
                chang=false;
                draw(&mut flgs,&mut known,&mut board,(0,0),false);
                //thread::sleep_ms(100);
                match rx.try_recv(){
                    Ok(_) => break,
                    Err(_) => continue,
                }
            }
        }
    });
    let stdin = io::stdin();
    match stdin.keys().next(){
        Some(_) => {
            tx.send(true);
            t.join();
            let mut stdout = io::stdout().into_raw_mode().unwrap();
            write!(stdout,"{}{}",termion::clear::All,termion::cursor::Goto(1,1));
            stdout.flush();
            return;
        },
        None => {;},
    }
}

fn player(){
    let mut correct=0;
    let mut playing = true;
    while playing{
        correct=0;
        playing = false;
        let mut fTurn=true;
        let mut flgs = [[false;LENGTH];HEIGHT];
        let mut known = [[false;LENGTH];HEIGHT];
        let mut board = [[0;LENGTH];HEIGHT];
        let mut stdout = io::stdout().into_raw_mode().unwrap();
        draw(&mut flgs,&mut known,&mut board,(0,0),false);
        if playing{continue;}
        let stdin = io::stdin();
        for c in stdin.keys(){
            match c.unwrap(){
                Key::Char(' ') => { //opens tile up
                    let (j,i) = stdout.cursor_pos().unwrap();
                    let (i,j) = ((i-1-IBORDR) as usize,(j-1-JBORDR) as usize);
                    if click(&mut flgs,&mut known,&mut board,&mut fTurn, (i,j)){
                        playing =  gameOver(&mut flgs,&mut known,&mut board,"you have lost."); //if this returns true, restart game
                        break;}
                },
                Key::Char('q') => {write!(stdout,"{}{}",termion::clear::All,termion::cursor::Goto(1,1));playing=false;break;}, //quits game
                Key::Char('f') => { //flag a bomb, allows for unflagging the same way
                    let (j,i) = stdout.cursor_pos().unwrap();
                    if (i > IBORDR && j> JBORDR){
                        let (i,j) = ((i-1-IBORDR) as usize,(j-1-JBORDR) as usize);
                        let mut c:char;
                        if (i < HEIGHT && j < LENGTH){
                            playing = placeFlag(&mut flgs,&mut known, &mut board,i,j,&mut correct);
                        }
                    }
                },
                Key::Up => {write!(stdout,"{}",termion::cursor::Up(1));}
                Key::Right => {write!(stdout,"{}",termion::cursor::Right(1));}
                Key::Down => {write!(stdout,"{}",termion::cursor::Down(1));}
                Key::Left => {write!(stdout,"{}",termion::cursor::Left(1));}
                _ => continue,
            }
            if playing{break;}
            stdout.flush().unwrap();
        }
    }
}

fn main() {
    if BOT{
        runBot();
    }
    else{player();}
}
