use std::collections::LinkedList;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::time::Instant;
use std::io;

#[derive(Debug, Copy, Clone)]
pub struct Cell {
    pub value: u8,
    pub options: [bool; 9]
}

pub struct ValueEntry {
    pub x: u8,
    pub y: u8,
    pub options: [bool; 9],
}

pub struct OptionEntry {
    pub x: u8,
    pub y: u8,
    pub option: u8,
}

pub enum JournalEntry {
    Value(ValueEntry),
    Option(OptionEntry),
}

pub enum RemoveResult{
    InvalidBoard,
    SingleOption(u8),
    LogJournal,
    DoNothing
}

struct SetInstruction{
    pub x: u8,
    pub y: u8,
    pub value: u8
}

impl Cell {
    pub fn remove_option(&mut self, option:u8) -> RemoveResult {
        if self.value == option as u8 {
            return RemoveResult::InvalidBoard
        }

        if self.value > 0 {
            return RemoveResult::DoNothing
        }

        if self.options[(option - 1) as usize] == true {
            self.options[(option - 1) as usize] = false;

            let mut count = 0;
            let mut value:usize = 0;

            for i in 0..9 {
                if self.options[i] {
                    count = count + 1;
                    value = i
                }
            }

            if count == 1 {
                return RemoveResult::SingleOption((value + 1) as u8)
            } else {
                return RemoveResult::LogJournal
            }
        }

        RemoveResult::DoNothing
    }

    pub fn set_value(&mut self, value:u8) -> Option<[bool;9]>{
        let mut add_journal:Option<[bool;9]> = Option::None;

        if self.value == 0 {
            add_journal = Option::Some(self.options);
        }

        for i in 0..9 {
            self.options[i] = false
        }

        self.value = value;

        return add_journal
    }
}

struct CompleteGame{
    journal:LinkedList<JournalEntry>,
    board:[[Cell;9];9],
    // solutions:Vec<[[u8;9];9]>
    solution_count: u64,
    last_update: Instant

}

#[derive(Debug)]
struct MinOptions{
    x: u8,
    y: u8,
    options: Vec<u8>
}

impl CompleteGame {
    pub fn new() -> Self {
        CompleteGame{
            journal: LinkedList::new(),
            board:[[
                Cell {
                    value: 0,
                    options: [true;9]
                }; 9]; 9],
            //solutions:Vec::new()
            solution_count: 0,
            last_update: Instant::now()
        }
    }

    fn board_solved(&mut self) -> bool {
        for x in 0 .. 9 {
            for y in 0 .. 9 {
                if self.board[x][y].value == 0 {
                    return false
                }
            }
        }

        true
    }

    fn min_options(&mut self) -> MinOptions {
        let mut min_options = 9;
        let mut min_x = 0;
        let mut min_y = 0;

        for x in 0 .. 9 {
            for y in 0 .. 9 {
                if self.board[x][y].value == 0 {
                    let mut cur_options = 0;

                    for n in 0 .. 9 {
                        if self.board[x][y].options[n] {
                            cur_options += 1
                        }
                    }

                    if cur_options < min_options {
                        min_options = cur_options;
                        min_x = x;
                        min_y = y;
                    }
                }
            }
        }

        let mut options:Vec<u8> = Vec::new();

        for n in 0 .. 9 {
            if self.board[min_x][min_y].options[n] {
                options.push((n + 1) as u8)
            }
        }

        MinOptions{x: min_x as u8, y: min_y as u8, options}
    }

    pub fn apply(&mut self, initial_set:Vec<SetInstruction>, level:u8){
        let mut to_set:LinkedList<SetInstruction> = LinkedList::new();

        for si in initial_set{
            to_set.push_back(si);
        }

        while !to_set.is_empty() {

            let si = to_set.pop_front().unwrap();

            match self.board[si.x as usize][si.y as usize].set_value(si.value) {
                Option::None => {}
                Option::Some(options) => {
                    // I only care if this operation results in action change
                    self.journal.push_back(JournalEntry::Value(ValueEntry { x: si.x, y: si.y, options }));

                    // This is mainly a shortcut to avoid repeating those lines for every restriction check
                    // it's important for it to be here since I want to borrow to_set as mutable only on this
                    // level
                    let mut remove_option = |x:u8, y:u8, option:u8| -> bool{
                        match self.board[x as usize][y as usize].remove_option(option) {
                            RemoveResult::DoNothing => {}
                            RemoveResult::InvalidBoard => { return true; }
                            RemoveResult::LogJournal => {
                                self.journal.push_back(JournalEntry::Option(OptionEntry{x,y,option}));
                            }
                            RemoveResult::SingleOption(value) => {
                                self.journal.push_back(JournalEntry::Option(OptionEntry{x,y,option}));
                                to_set.push_back(SetInstruction{x, y, value});
                            }
                        }

                        false
                    };


                    for scan_x in 0 .. 9 {
                        if scan_x != si.x {
                            if remove_option(scan_x as u8, si.y, si.value) {
                                return;
                            }
                        }
                    }

                    for scan_y in 0 .. 9 {
                        if scan_y != si.y {
                            if remove_option(si.x, scan_y as u8, si.value) {
                                return;
                            }
                        }
                    }

                    let sector_x = si.x / 3;
                    let sector_y = si.y / 3;

                    for scan_x in sector_x * 3 .. (sector_x + 1) * 3 {
                        for scan_y in sector_y * 3 .. (sector_y + 1) * 3 {
                            if scan_x != si.x || scan_y != si.y {
                                if remove_option(scan_x, scan_y as u8, si.value) {
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }

        if self.board_solved() {
            let mut result:[[u8;9];9] = [[0;9];9];

            for x in 0 .. 9 {
                for y in 0..9 {
                    result[x][y] = self.board[x][y].value;
                }
            }

            // self.solutions.push(result);
            self.solution_count += 1;

            if self.solution_count % 100000 == 0 {
                let elapsed = self.last_update.elapsed();
                if elapsed.as_secs() > 10 {
                    self.last_update = Instant::now();
                    println!("count {}", self.solution_count);
                }
            }
        } else {

            let journal_pos = self.journal.len();

            let min_options = self.min_options();

            for opt in min_options.options {
                self.apply(vec![SetInstruction{x:min_options.x, y:min_options.y, value:opt}], level+1);

                while journal_pos < self.journal.len() {
                    match self.journal.pop_back().unwrap() {
                        JournalEntry::Option(revert_option) => {
                            self.board[revert_option.x as usize][revert_option.y as usize].options[(revert_option.option - 1) as usize] = true;
                        }
                        JournalEntry::Value(revert_value) => {
                            self.board[revert_value.x as usize][revert_value.y as usize].value = 0;
                            self.board[revert_value.x as usize][revert_value.y as usize].options = revert_value.options;
                        }
                    }
                }
            }


        }


    }
}

fn parse_file(file_name:&str) -> Result<Vec<SetInstruction>, io::Error> {
    let fh = File::open(file_name)?;

    let mut initial_instructions:Vec<SetInstruction> = Vec::new();

    let file = BufReader::new(&fh);
    let mut y = 0;

    for line in file.lines() {
        let s = line.unwrap();
        let l = s.into_bytes();

        for x in 0..9 {
            if x < l.len() && l[x] != 32 {
                initial_instructions.push(SetInstruction{x:x as u8,y:y as u8,value:l[x]-48})
            }
        }

        y += 1;
    }

    Ok(initial_instructions)
}

fn main() -> Result<(),io::Error> {
    println!("Hello, world!");

    /*let journal: LinkedList<JournalEntry> = LinkedList::new();
    let board = [[
        Cell {
            value: 0,
            options: [true, true, true, true, true, true, true, true, true]
        }; 9]; 9];

    board_apply(board, journal, Vec::new(), 0);*/



    let start = Instant::now();

    let mut cg = CompleteGame::new();

    let initial = parse_file("boards/hard-1")?;
    // let initial = vec![];
    cg.apply(initial, 0);

    // println!("solutions: {}", cg.solutions.len());
    println!("solutions: {}", cg.solution_count);

    //println!("{:?}", cg.solutions);
    let elapsed = start.elapsed();
    let sec = (elapsed.as_secs() as f64) + (elapsed.subsec_nanos() as f64 / 1000_000_000.0);
    println!("{} seconds elapsed.", sec);

    Ok(())
}
