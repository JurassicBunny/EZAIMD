use std::fs::File;
use std::io::Read;

use crate::vectored::{Force, Position, Vectored, Velocity};
use anyhow::{Context, Result};
use rand_distr::{Distribution, Normal};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    pub symbol: String,
    pub mass: f64,
    pub can_mv: bool,
    pub pos: Position<f64>,
    pub vel: Velocity<f64>,
    pub force: Force<f64>,
    pub next_force: Force<f64>,
}

#[derive(Debug)]
pub struct AtomFactory {
    file: File,
}

impl AtomFactory {
    pub fn new(file: File) -> AtomFactory {
        AtomFactory { file }
    }

    pub fn gn_atoms(mut self) -> Result<Vec<Atom>> {
        let mut buffer = String::new();
        self.file.read_to_string(&mut buffer).unwrap();
        let data = AtomFactory::read_atomic_lines(buffer.clone())
            .with_context(|| format!("Failed to read atomic data from input"))?;

        let num_atoms = AtomFactory::get_num_atoms(buffer.clone());
        println!("{}", num_atoms.clone());
        let atomic_lines = data
            .into_iter()
            .rev()
            .take(num_atoms)
            .rev()
            .collect::<Vec<String>>();

        for line in atomic_lines.clone() {
            println!("{}", line);
        }

        let atoms = atomic_lines
            .into_iter()
            .map(|x| Self::make_atom(x))
            .collect::<Vec<Atom>>();
        let result = Self::rm_cmv(atoms);

        Ok(result)
    }

    fn read_atomic_lines(buffer: String) -> Result<Vec<String>> {
        let to_find = Regex::new(r"^(\s)+\d+(\s)+\d+(\s)+\d+((\s+)-?\d+.\d+){3}").unwrap();
        let result = buffer
            .lines()
            .filter(|x| to_find.is_match(x))
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
        Ok(result)
    }

    fn get_num_atoms(buffer: String) -> usize {
        let to_find = Regex::new(r"NAtoms=").unwrap();
        let result = buffer
            .lines()
            .filter(|x| to_find.is_match(x))
            .map(|x| x.to_owned())
            .take(1)
            .collect::<String>()
            .split_whitespace()
            .find_map(|x| x.parse::<usize>().ok());

        result.unwrap()
    }

    fn make_atom(line: String) -> Atom {
        let mut split_line = line.split_whitespace();
        split_line.next().unwrap();
        let symbol_line = split_line.next().unwrap();
        let symbol_mass = Self::gn_symbol(symbol_line.parse::<u32>().unwrap()).unwrap();
        split_line.next().unwrap();
        let x = split_line.next().unwrap();
        let y = split_line.next().unwrap();
        let z = split_line.next().unwrap();
        let pos = Position::new(
            x.parse::<f64>().unwrap(),
            y.parse::<f64>().unwrap(),
            z.parse::<f64>().unwrap(),
        );

        let velocity = Self::rand_vel(symbol_mass.mass);
        let vel = Velocity::new(velocity, velocity, velocity);
        let force = Force::new(0.0, 0.0, 0.0);
        let next_force = Force::new(0.0, 0.0, 0.0);

        let result = Atom {
            symbol: symbol_mass.symbol,
            mass: symbol_mass.mass,
            can_mv: true,
            pos,
            vel,
            force,
            next_force,
        };

        result
    }

    fn gn_symbol(num: u32) -> Result<SymbolMass> {
        let result = match num {
            1 => Ok(SymbolMass::new("H", 1.008)),
            2 => Ok(SymbolMass::new("He", 4.0026)),
            6 => Ok(SymbolMass::new("C", 12.011)),
            7 => Ok(SymbolMass::new("N", 14.007)),
            8 => Ok(SymbolMass::new("O", 15.999)),
            9 => Ok(SymbolMass::new("F", 18.998)),
            10 => Ok(SymbolMass::new("Ne", 20.180)),
            15 => Ok(SymbolMass::new("P", 30.974)),
            16 => Ok(SymbolMass::new("S", 32.06)),
            17 => Ok(SymbolMass::new("Cl", 35.45)),
            47 => Ok(SymbolMass::new("Ag", 107.87)),
            79 => Ok(SymbolMass::new("Au", 196.97)),
            _ => Err(anyhow::anyhow!("atomic number: {}, is not supported!", num)),
        };
        result
    }

    fn rand_vel(mass: f64) -> f64 {
        //m^2*kg*s^-2*K^-1
        let boltzmann = 1.380649e-23f64;
        //K
        let Temp = 300.0;
        //kg
        let new_mass = mass * (1.0 / 6.0221408e23f64) * (1.0 / 1000.0);
        //A^2/fs^2
        let var = ((boltzmann * Temp) / new_mass) * 10e-10f64;
        let normal = Normal::new(0.0, var.sqrt()).unwrap();
        let value = normal.sample(&mut rand::thread_rng());
        value
    }

    fn rm_cmv(atoms: Vec<Atom>) -> Vec<Atom> {
        let cmv_mv: Velocity<f64> = atoms
            .clone()
            .into_iter()
            .map(|x| x.mass * x.vel)
            .fold(Velocity::new(0.0, 0.0, 0.0), |a, b| a + b);
        let cmv_m: f64 = atoms.clone().into_iter().map(|x| x.mass).sum();
        let cmv = cmv_mv * (1.0 / cmv_m);
        let result = atoms
            .into_iter()
            .map(|x| Self::apply_to_atom(x, cmv))
            .collect::<Vec<Atom>>();
        result
    }

    fn apply_to_atom(mut atom: Atom, value: Velocity<f64>) -> Atom {
        atom.vel = atom.vel - value;
        atom
    }
}

struct SymbolMass {
    symbol: String,
    mass: f64,
}

impl SymbolMass {
    fn new(symbol: &'static str, mass: f64) -> Self {
        SymbolMass {
            symbol: symbol.to_string(),
            mass,
        }
    }
}
