use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader};
use std::io::{Read, Write};
use std::path::Path;

use crate::vectored::{Force, Position, Vectored, Velocity};
use anyhow::Result;
use regex::Regex;
use rgaussian16::Gaussian;
use serde::{Deserialize, Serialize};

use crate::atom::{Atom, AtomFactory};
use crate::cli::Args;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Simulation {
    atoms: Vec<Atom>,
    time_step: f64,
    num_steps: usize,
    step_num: usize,
    pot_energy: f64,
    kin_energy: f64,
    tot_energy: f64,
}

impl Simulation {
    pub fn new(args: &Args) -> Result<Simulation> {
        let file = File::open(&args.input)?;
        let time_step = args.time_step;
        let num_steps = args.num_steps;
        let mut atoms = AtomFactory::new(file).gn_atoms()?;
        if let Some(value) = &args.freeze {
            Self::validate_string(value.to_owned())?;
            atoms = Self::freeze_atoms(&atoms, value.to_owned());
        };

        atoms
            .clone()
            .into_iter()
            .filter(|x| x.can_mv == false)
            .for_each(|x| println!("Atom: {} is frozen", x.symbol));

        Ok(Simulation {
            atoms,
            time_step,
            num_steps,
            step_num: 0,
            pot_energy: 0.0,
            kin_energy: 0.0,
            tot_energy: 0.0,
        })
    }

    pub fn run(mut self) {
        if self.step_num == 0 {
            Self::init_files();
            self.report_trajectory();
            self.report_energy();
            self.report_velocity();
            self.report_kinetic();
            self.save();
            self.step_num += 1;
        }
        while self.step_num <= self.num_steps {
            self.update_pos();
            self.generate_input();
            self.run_gaussian();
            let data = self.read_gaussian();
            self.update_next_forces(data.forces)
                .update_vel()
                .update_pot(data.scf)
                .update_kin()
                .update_tot();
            self.report_trajectory();
            self.report_energy();
            self.save();
            self.step_num += 1;
        }
    }

    pub fn init_forces(mut self) -> Self {
        self.generate_input();
        self.run_gaussian();
        let data = self.read_gaussian();
        self.update_forces(data.forces)
            .update_pot(data.scf)
            .update_kin()
            .scale_temp()
            .update_kin()
            .update_tot()
    }

    pub fn from_save() -> Simulation {
        let mut simulation: Simulation = Self::read_to_vec("save.json").last().unwrap().clone();
        simulation.step_num += 1;
        simulation
    }

    fn freeze_atoms(atoms: &Vec<Atom>, string: String) -> Vec<Atom> {
        let to_freeze = Self::parse_string(string);
        let mut atoms = atoms.clone();
        for value in to_freeze {
            atoms[(value - 1) as usize].can_mv = false;
            atoms[(value - 1) as usize].vel = Velocity::new(0.0, 0.0, 0.0);
        }
        atoms
    }

    fn validate_string(_string: String) -> Result<()> {
        Ok(())
    }

    fn parse_string(string: String) -> Vec<u32> {
        let ranges = string
            .split(',')
            .map(|x| Self::convert_to_range(x.to_string()).gen_numbers())
            .flatten()
            .collect::<Vec<u32>>();
        ranges
    }

    fn convert_to_range(line: String) -> Range {
        let result = line
            .split("-")
            .into_iter()
            .filter_map(|x| x.parse::<u32>().ok())
            .collect::<Vec<u32>>();
        Range::new(result[0], result[1])
    }

    fn read_to_vec<P>(path: P) -> Vec<Simulation>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path).expect("failed to open save.json");
        let result: Vec<Simulation> = BufReader::new(file)
            .lines()
            .into_iter()
            .map(|line| serde_json::from_str(&line.unwrap()).unwrap())
            .collect();
        result
    }

    fn save(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("save.json")
            .expect("failed to open save.json during report");
        let report =
            serde_json::to_string(&self).expect("unable to convert simulation into string");
        write!(file, "{}\n", report).expect("failed to write to save.json");
    }

    fn update_pos(&mut self) -> &mut Self {
        let function = |x: Atom| -> Position<f64> {
            if x.can_mv {
                x.pos
                    + (x.vel * self.time_step)
                    + (0.5 * x.force * (1.0 / x.mass) * self.time_step.powi(2))
            } else {
                x.pos
            }
        };
        let result = self
            .atoms
            .clone()
            .into_iter()
            .map(function)
            .collect::<Vec<Position<f64>>>();

        let mut index = 0;
        for value in result {
            self.atoms[index].pos = value;
            index += 1;
        }

        self
    }

    fn update_vel(&mut self) -> &mut Self {
        let function = |x: Atom| -> Velocity<f64> {
            if x.can_mv {
                x.vel + (0.5 * (x.force + x.next_force) * (1.0 / x.mass) * self.time_step)
            } else {
                x.vel
            }
        };

        let result = self
            .atoms
            .clone()
            .into_iter()
            .map(function)
            .collect::<Vec<Velocity<f64>>>();

        let mut index = 0;
        for value in result {
            self.atoms[index].vel = value;
            self.atoms[index].force = self.atoms[index].next_force;
            index += 1;
        }
        self
    }

    fn generate_input(&self) {
        let input = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open("input.com")
            .expect("failed to spawn input.com file");

        let coords = self.clone().gen_coords();

        let config = File::open("config.yaml").expect("failed to open config.yaml");
        let interface = Gaussian::new(config)
            .expect("failed to generate Gaussian16 interface. Check config.yaml");

        interface.gen_input(&input).expect("failed to write input");
        writeln!(&input, "\n{}\n", coords).expect("failed to write atomic coords");
    }

    fn gen_coords(self) -> String {
        let lines = self
            .atoms
            .into_iter()
            .map(|x| {
                format!(
                    "{} {:.5} {:.5} {:.5}",
                    x.symbol,
                    x.pos.as_vec().x,
                    x.pos.as_vec().y,
                    x.pos.as_vec().z
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        lines
    }

    fn run_gaussian(&self) {
        let input = File::open("input.com").expect("failed to open input.com for Gaussian16 run");
        let output = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open("forces.out")
            .expect("failed to create output file");

        let config = File::open("config.yaml").expect("failed to open config.yaml");
        let interface = Gaussian::new(config)
            .expect("failed to generate Gaussian16 interface. Check config.yaml");

        interface
            .run(input, output)
            .expect("Gaussian16 calculation failed");
    }

    fn read_gaussian(&self) -> GaussianOutput {
        let output = File::open("forces.out").expect("failed to open forces.out");
        GaussianOutput::new(output)
    }

    fn update_forces(&mut self, forces: Vec<Force<f64>>) -> &mut Self {
        let mut index = 0;
        for force in forces {
            self.atoms[index].force = force;
            index += 1;
        }
        self
    }

    fn update_next_forces(&mut self, forces: Vec<Force<f64>>) -> &mut Self {
        let mut index = 0;
        for force in forces {
            self.atoms[index].next_force = force;
            index += 1;
        }
        self
    }

    fn update_pot(&mut self, value: f64) -> &mut Self {
        self.pot_energy = (value * 2625.5) / 100.0;
        self
    }

    fn update_kin(&mut self) -> &mut Self {
        let value: f64 = self
            .atoms
            .clone()
            .into_iter()
            .map(|x| 0.5 * x.mass * x.vel.sqr_norm())
            .sum();
        self.kin_energy = value * 100.0;
        self
    }

    fn update_tot(&mut self) -> Self {
        let value = self.pot_energy + self.kin_energy;
        self.tot_energy = value;
        self.clone()
    }

    fn init_files() {
        InitFiles::new()
    }

    fn scale_temp(&mut self) -> &mut Self {
        let scalar = (300.0
            / ((2.0 / 3.0) * ((self.kin_energy * 100.0 * 1000.0) / 8.31446261815324)))
            .sqrt();
        let mut index = 0;
        let length = self.atoms.clone();
        for _atom in length {
            self.atoms[index].vel = self.atoms[index].vel * scalar;
            index += 1;
        }
        self
    }

    fn report_trajectory(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("trajectory.xyz")
            .expect("failed to report trajectory");
        let to_write = format!(
            "{}\ntrjectory\n{}\n",
            self.atoms.len(),
            self.clone().gen_coords()
        );
        file.write(to_write.as_bytes())
            .expect("you managed the imposable");
    }

    fn report_energy(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("energy.txt")
            .expect("failed to report energy");
        let to_write = format!(
            "{:<30.2} {:<30.6} {:<30.6} {:.6}\n",
            self.step_num as f64 * self.time_step,
            self.pot_energy,
            self.kin_energy,
            self.tot_energy
        );
        file.write(to_write.as_bytes())
            .expect("you managed the imposable");
    }

    fn report_velocity(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("velocity.txt")
            .expect("failed to report velocity");
        let mut to_write: Vec<String> = vec![];
        let mut index = 1;
        for atom in self.atoms.clone() {
            let string = format!(
                "{:<30} {:<30} {:<30} {:<30} {:<30} {}",
                index,
                atom.symbol,
                atom.vel.as_vec().x,
                atom.vel.as_vec().y,
                atom.vel.as_vec().z,
                atom.vel.norm()
            );
            to_write.push(string);
            index += 1;
        }

        let value = to_write.join("\n");
        file.write(value.as_bytes())
            .expect("you managed the imposable");
    }

    fn report_kinetic(&self) {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("kinetic.txt")
            .expect("failed to report kinetic");
        let mut to_write: Vec<String> = vec![];
        let mut index = 1;
        for atom in self.atoms.clone() {
            let string = format!(
                "{:<30} {:<30} {}",
                index,
                atom.symbol,
                (0.5 * atom.mass * atom.vel.sqr_norm() * 100.0)
            );
            to_write.push(string);
            index += 1;
        }

        let value = to_write.join("\n");
        file.write(value.as_bytes())
            .expect("you managed the imposable");
    }
}

struct InitFiles {}

impl InitFiles {
    fn new() {
        Self::init_energy();
        Self::init_kinetic();
        Self::init_velocity();
        Self::init_trajectory();
        Self::init_save();
    }

    fn init_energy() {
        let init_string = format!(
            "{:<30} {:<30} {:<30} {}\n",
            "Time fs", "Potential 100 KJ/mol", "Kinetic 100 KJ/mol", "Total 100 KJ/mol"
        );

        Self::generate("energy.txt", init_string);
    }

    fn init_kinetic() {
        let init_string = format!(
            "{:<30} {:<30} {}\n",
            "Number", "Symbol", "Kinetic 100 kJ/mol"
        );

        Self::generate("kinetic.txt", init_string);
    }

    fn init_velocity() {
        let init_string = format!(
            "{:<30} {:<30} {:<30} {:<30} {:<30} {}\n",
            "Number", "Symbol", "X", "Y", "Z", "Magnitude"
        );
        Self::generate("velocity.txt", init_string);
    }

    fn init_trajectory() {
        Self::generate("trajectory.xyz", "".to_string());
    }

    fn init_save() {
        Self::generate("save.json", "".to_string());
    }

    fn generate(name: &str, init_string: String) {
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&name)
            .expect(format!("failed to init: {} file", name).as_str());
        file.write(init_string.as_bytes()).unwrap();
    }
}

struct GaussianOutput {
    scf: f64,
    forces: Vec<Force<f64>>,
}

impl GaussianOutput {
    pub fn new(mut file: File) -> GaussianOutput {
        let mut buffer = String::new();
        let to_find = Regex::new(r"^(\s)+\d+(\s)+\d+((\s+)-?\d+.\d+){3}").unwrap();
        let to_find_scf = Regex::new(r"^ SCF Done").unwrap();
        file.read_to_string(&mut buffer).unwrap();

        let forces = buffer
            .clone()
            .lines()
            .filter(|x| to_find.is_match(x))
            .map(|x| x.to_string())
            .map(|x| Self::convert_to_force(x))
            .collect::<Vec<Force<f64>>>();

        let scf = buffer
            .lines()
            .filter(|x| to_find_scf.is_match(x))
            .map(|x| x.to_string())
            .rev()
            .take(1)
            .collect::<String>()
            .split_whitespace()
            .into_iter()
            .find_map(|x| x.parse::<f64>().ok())
            .unwrap();

        GaussianOutput { scf, forces }
    }

    fn convert_to_force(line: String) -> Force<f64> {
        let result = line
            .split_whitespace()
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        let force = Force::new(
            result[2].parse::<f64>().unwrap(),
            result[3].parse::<f64>().unwrap(),
            result[4].parse::<f64>().unwrap(),
        );

        //convert form Eh/Bohr to Ag/mol*fs^2
        force * 0.496147792
    }
}

struct Range {
    low: u32,
    high: u32,
}

impl Range {
    fn new(low: u32, high: u32) -> Range {
        Range { low, high }
    }

    fn gen_numbers(&self) -> Vec<u32> {
        let result: Vec<u32> = (self.low..=self.high).collect();
        result
    }
}
