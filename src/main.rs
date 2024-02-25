use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::process;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Block {
    Academics,
    Culture,
    Fest,
    Community,
    Experience,
}

impl Block {
    const VALUES: [Block; 5] = [
        Block::Academics,
        Block::Culture,
        Block::Fest,
        Block::Community,
        Block::Experience,
    ];
}

impl<'de> Deserialize<'de> for Block {
    fn deserialize<D>(deserializer: D) -> Result<Block, D::Error>
    where
        D: Deserializer<'de>,
    {
        let n = u8::deserialize(deserializer)?;
        match n {
            1 => Ok(Block::Academics),
            2 => Ok(Block::Culture),
            3 => Ok(Block::Fest),
            4 => Ok(Block::Community),
            5 => Ok(Block::Experience),
            _ => Err(serde::de::Error::custom("unknown block")),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct Program {
    name: String,
    block: Block,
    min_cap: u8,
    max_cap: u8,
}

fn load_programs() -> Result<Vec<Program>, Box<dyn Error>> {
    let file = File::open("src/programs.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut programs = vec![];
    for result in rdr.deserialize() {
        let program: Program = result?;
        programs.push(program);
    }
    Ok(programs)
}

#[derive(Debug)]
struct Student {
    name: String,
    prefs: HashMap<Block, Preference>,
}

impl Student {
    fn from_record(record: csv::StringRecord, programs: &Vec<Program>) -> Option<Student> {
        let mut prefs = HashMap::new();

        for (i, block) in Block::VALUES.iter().enumerate() {
            let pref = Preference {
                first: programs
                    .iter()
                    .find(|p| p.name == record[(i * 5) + 0 + 6])?
                    .clone(),
                second: programs
                    .iter()
                    .find(|p| p.name == record[(i * 5) + 1 + 6])?
                    .clone(),
                third: programs
                    .iter()
                    .find(|p| p.name == record[(i * 5) + 2 + 6])?
                    .clone(),
                fourth: programs
                    .iter()
                    .find(|p| p.name == record[(i * 5) + 3 + 6])?
                    .clone(),
                fifth: programs
                    .iter()
                    .find(|p| p.name == record[(i * 5) + 4 + 6])?
                    .clone(),
            };
            prefs.insert(block.clone(), pref);
        }

        Some(Student {
            name: format!("{} {}", &record[3], &record[2]),
            prefs,
        })
    }
}

#[derive(Debug)]
struct Preference {
    first: Program,
    second: Program,
    third: Program,
    fourth: Program,
    fifth: Program,
}

fn load_students(programs: &Vec<Program>) -> Result<Vec<Student>, Box<dyn Error>> {
    let file = File::open("src/signups.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut students = vec![];
    for record in rdr.records() {
        let student = Student::from_record(record?, programs).ok_or("Invalid student")?;
        students.push(student);
    }
    Ok(students)
}

struct App {
    programs: Vec<Program>,
    students: Vec<Student>,
}

impl App {
    fn new() -> Result<App, Box<dyn Error>> {
        let programs = load_programs()?;
        let students = load_students(&programs)?;
        Ok(App { programs, students })
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
