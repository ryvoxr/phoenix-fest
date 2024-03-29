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
    room_num: u32,
    min_cap: u8,
    max_cap: u8,
    #[serde(skip)]
    students: Vec<StudentId>,
    #[serde(skip)]
    id: ProgramId,
}

impl Program {
    fn is_full(&self) -> bool {
        self.students.len() as u8 >= self.max_cap
    }

    fn student_is_assigned(&self, studentid: StudentId) -> bool {
        self.students.contains(&studentid)
    }
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
    assignments: HashMap<Block, ProgramId>,
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
            assignments: HashMap::new(),
        })
    }

    fn get_pref_program(&self, block: &Block, prefn: usize) -> Option<ProgramId> {
        *self.prefs.get(block)?.get(prefn)?
    }

    fn is_assigned(&self, block: &Block) -> bool {
        self.assignments.contains_key(block)
    }

    fn get_assignment_score(&self) -> Option<u8> {
        let mut score = 0;
        for block in Block::VALUES.iter() {
            let program_id = match self.assignments.get(block) {
                Some(p) => p,
                None => break,
            };
            let pref = self.prefs.get(block)?;
            let mut requested = false;
            for (i, p) in pref.iter().enumerate() {
                match p {
                    None => continue,
                    Some(p) => {
                        if p == program_id {
                            score += i as u8;
                            requested = true;
                            break;
                        }
                    }
                }
            }
            if !requested {
                score += 10;
            }
        }
        Some(score)
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

    fn assign(&mut self) -> Result<(), Box<dyn Error>> {
        for block in Block::VALUES.iter() {
            for prefn in 0..5 {
                let student_ids = self.student_id_list_sorted();
                for student_id in student_ids {
                    let student = &mut self.students[student_id];
                    if student.is_assigned(block) {
                        continue;
                    }
                    let programid = match student.get_pref_program(block, prefn) {
                        Some(p) => p,
                        None => continue,
                    };
                    let program = &mut self.programs[programid];
                    if program.is_full() {
                        continue;
                    }
                    program.students.push(student.id);
                    student.assignments.insert(block.clone(), programid);
                }
            }
            // assign unassigned students to least full programs
            for student_id in 0..self.students.len() {
                if self.students[student_id].is_assigned(block) {
                    continue;
                }
                self.no_pref_assign(block, student_id);
            }
        }
        Ok(())
    }

    fn no_pref_assign(&mut self, block: &Block, student_id: StudentId) {
        let program_ids = self.program_id_list_sorted(block);
        let student = &mut self.students[student_id];
        for program_id in program_ids {
            let program = &mut self.programs[program_id];
            if program.is_full() {
                continue;
            }
            program.students.push(student.id);
            student.assignments.insert(block.clone(), program_id);
            return;
        }
    }

    // sort program ids by how many students are assigned to them, filter by block
    fn program_id_list_sorted(&self, block: &Block) -> Vec<ProgramId> {
        let mut program_ids: Vec<ProgramId> = (0..self.programs.len()).filter(|&id| {
            self.programs[id].block == *block
        }).collect();
        program_ids.sort_by(|a, b| {
            let program_a = &self.programs[*a];
            let program_b = &self.programs[*b];
            program_a.students.len().cmp(&program_b.students.len())
        });
        program_ids
    }

    // sort student ids by assignment score (greatest first) (greater is worse assignment)
    fn student_id_list_sorted(&self) -> Vec<StudentId> {
        let mut student_ids: Vec<StudentId> = (0..self.students.len()).collect();
        student_ids.sort_by(|a, b| {
            let student_a = &self.students[*a];
            let student_b = &self.students[*b];
            student_a.get_assignment_score().cmp(&student_b.get_assignment_score())
        });
        student_ids.reverse();
        student_ids
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut app = App::new()?;
    app.assign()?;

    for student in app.students.iter() {
        println!("{}: {}", student.name, student.get_assignment_score().unwrap());
        for (block, program_id) in student.assignments.iter() {
            let program = &app.programs[*program_id];
            println!("  {:?}: {}", block, program.name.trim_start_matches(char::is_numeric).trim_start_matches(". "));
        }
    }

    println!("{:?}", app.students[*app.student_id_list_sorted().first().unwrap()].get_assignment_score());

    // for program in app.programs.iter() {
        // println!("{}:", program.name);
        // for student_id in program.students.iter() {
            // let student = &app.students[*student_id];
            // println!("  {}", student.name);
        // }
    // }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
