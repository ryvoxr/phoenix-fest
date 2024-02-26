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

#[derive(Debug, Clone, Deserialize)]
struct Program {
    name: String,
    block: Block,
    min_cap: u8,
    max_cap: u8,
    #[serde(skip)]
    students: Vec<StudentId>,
    #[serde(skip)]
    id: ProgramId,
}

fn load_programs() -> Result<Vec<Program>, Box<dyn Error>> {
    let file = File::open("src/programs.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut programs = vec![];
    for (id, result) in rdr.deserialize().enumerate() {
        let mut program: Program = result?;
        program.id = id;
        programs.push(program);
    }
    Ok(programs)
}

#[derive(Debug)]
struct Student {
    name: String,
    prefs: HashMap<Block, Preference>,
    id: StudentId,
}

type StudentId = usize;
type ProgramId = usize;

impl Student {
    fn from_record(
        record: &csv::StringRecord,
        programs: &Vec<Program>,
        id: StudentId,
    ) -> Option<Student> {
        let mut prefs = HashMap::new();

        for (i, block) in Block::VALUES.iter().enumerate() {
            let mut pref = vec![];
            for j in 0..5 {
                match programs.iter().find(|p| p.name == record[(i * 5) + j + 6]) {
                    Some(p) => pref.push(Some(p.id)),
                    None => pref.push(None),
                }
            }
            prefs.insert(block.clone(), pref);
        }

        Some(Student {
            name: format!("{} {}", &record[3], &record[2]),
            prefs,
            id,
        })
    }
}

type Preference = Vec<Option<ProgramId>>;

fn load_students(programs: &Vec<Program>) -> Result<Vec<Student>, Box<dyn Error>> {
    let file = File::open("src/signups.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut students = vec![];
    for (id, record) in rdr.records().enumerate() {
        let record = record?;
        let student = Student::from_record(&record, programs, id.try_into()?)
            .ok_or(format!("Invalid student: {}", &record[1]))?;
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

    fn assign_students(&mut self) -> Result<(), Box<dyn Error>> {
        for prefn in 0..5 {
            for block in Block::VALUES.iter() {
                for student in &self.students {
                    let pref = student.prefs.get(block).ok_or("Invalid block")?;
                    let program = pref[prefn];
                    match program {
                        Some(p) => {}
                        None => {}
                    }
                }
            }
        }
        Ok(())
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let app = App::new()?;
    for program in &app.programs {
        println!("{:?}", program);
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
