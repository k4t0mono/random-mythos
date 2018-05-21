// An attempt to make a procedural Mythos generator
#![allow(dead_code)]
#[macro_use]
extern crate log;
extern crate simplelog;
extern crate rand;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
extern crate docopt;

use rand::Rng;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io;
use docopt::Docopt;

pub fn write_file<'a>(data: &'a str, path: &'a str) {
	let f = OpenOptions::new()
			.write(true)
			.create(true)
			.open(path);

	let mut file = match f {
		Err(e) => {
			error!("Something is terrible wrong happend while oppening the file");
			error!("{}", e);

			panic!(e)
		},

		Ok(fl) => fl,
	};
	
	match file.write_all(data.as_bytes()) {
		Err(e) => {
			error!("Something is terrible wrong happend while writing the file");
			error!("{}", e);

			panic!(e)
		},

		Ok(_) => info!("File {} writed sucessfully", path),
	}
}

fn load_file<'a>(path: &'a str) -> String {
    let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path);

    let mut file = match f {
        Err(e) => {
            error!("Something is terrible wrong happend while oppening the file");
            error!("{}", e);

            panic!(e)
        },

        Ok(fl) => fl,
    };

    let mut data = String::new();
    match file.read_to_string(&mut data) {
        Err(e) => {
            error!("Something is terrible wrong happend while reading the file");
            error!("{}", e);

            panic!(e)
        },

        Ok(_) => info!("File opened"),
    }
    
    data
}


trait Dot {
    fn to_dot(&self) -> String;
}


#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
struct Entity {
	name: String,
	level: i8,
}

impl Dot for Entity {
	fn to_dot(&self) -> String {
		format!(
			"{0} [label=\"{{ {0} | {1} }}\"]",
			self.name,
			self.level,
		)
	}
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
enum RelationType {
	Base,
	Parent,
	Invoker,
	Creator,
}


#[derive(Deserialize, Serialize)]
struct Relations {
	entites: Vec<Entity>,
	relations: Vec<Vec<Option<RelationType>>>,
}

impl Relations {
	pub fn init(size: usize) -> Relations {
		let mut entites: Vec<Entity> = vec![];
		for i in 0..size {
			entites.push(Entity{
				name: format!("ent{:02}", i),
				level: 0,
			});
		}

		let relations: Vec<Vec<Option<RelationType>>> = vec![vec![None; size]; size];

		Relations{ entites, relations }
	}

    pub fn from_json(json: String) -> Relations {
        let r: Relations = serde_json::from_str(&json).unwrap();

        r
    }

	fn add(&mut self, source: usize, destiny: usize, rt : RelationType) {
		self.relations[source][destiny] = Some(rt);
	}

	fn adjacent_out(&self, vertex: usize) -> Vec<usize> {
		let mut v: Vec<usize> = vec![];
		
		for j in 0..self.entites.len() {
			match &self.relations[vertex][j] {
				&Some(_) => v.push(j),
				&None => (),
			}
		}

		v
	}

	fn adjacent_in(&self, vertex: usize) -> Vec<usize> {
		let mut v: Vec<usize> = vec![];
		
		for i in 0..self.entites.len() {
			match &self.relations[i][vertex] {
				&Some(_) => v.push(i),
				&None => (),
			}
		}

		v
	}

	fn generate_base_relation(&mut self) {
		let s = self.entites.len();
		trace!("self.entites: {}", s);

		let n = s as usize;
		trace!("num relations: {}", n);

		for _i in 0..n {
			let mut src = rand::thread_rng().gen_range(0, s);
			let mut dest = rand::thread_rng().gen_range(0, s);
			trace!("src: {:?}, dest: {:?}", src, dest);
			
			while src == dest  {
				trace!("I can't be my own src");
				dest = rand::thread_rng().gen_range(0, s);
				trace!("New dest: {}", dest);
			}
			
			while let &Some(ref _rt) = &self.relations[src][dest] {
				trace!("You already are the src");
				src = rand::thread_rng().gen_range(0, s);
				trace!("New src: {}", src);
			}

			let mut stack: Vec<usize> = vec![];
			let mut verif = vec![false; n];
			stack.push(dest);
			trace!("Verificando ciclos");
			while let Some(top) = stack.pop() {
				trace!("v: {}", top);

				if verif[top] || top == src {
					trace!("A cicle identifyed");

					stack.clear();
					verif = vec![false; n];
					dest = rand::thread_rng().gen_range(0, s);
					stack.push(dest);

					trace!("New dest: {}",dest);

				} else {
					verif[top] = true;
					let adj = self.adjacent_out(top);
					for i in adj.iter() { stack.push(*i); }
				}
			}

			info!("src: {:?}, dest: {:?}", src, dest);
			self.add(src, dest, RelationType::Base);
		}
	}

	fn generate_relations(&mut self) {
		let n = self.entites.len();
		info!("n relations: {}", n);

		for e in 0..n {
			info!("ent: {}", self.entites[e].name);
			let adj_in = self.adjacent_in(e);
			trace!("adj_in: {:?}", adj_in);

			let rt_n = rand::thread_rng().gen_range(0, 3);

			let rt: RelationType = match rt_n {
				0 => RelationType::Parent,
				1 => RelationType::Invoker,
				2 => RelationType::Creator,
				
				_ => panic!("Help"),
			};

			info!("rt: {:?}", rt);
			for src in adj_in.iter() {
				trace!("src {}", src);
				self.relations[*src][e] = Some(rt);
			}
		}
	}

    fn entity_description(&self, e: usize) -> String {
        let get_names = |v: Vec<usize>| -> String {
            let n = v.len();
            let mut s: String = "".to_string();
            
            s += &self.entites[v[0]].name;

            if n == 2 {
                s += &format!(" and {}", &self.entites[v[0]].name);

            } else if n > 2 {
                for i in 1..n-1 {
                    s += &format!(", {}", &self.entites[v[i]].name);
                }

                s += &format!(" and {}", &self.entites[v[n-1]].name);
            }

            s
        };

        let mut s: String = format!("{}", self.entites[e].name);

        let adj_in = self.adjacent_in(e);

        if adj_in.len() == 0 {
            s += " children of the Void";

        } else {
            let rt = self.relations[adj_in[0]][e].unwrap();

            s += match rt {
                RelationType::Invoker => " invoked by",
                RelationType::Creator => " created by",
                RelationType::Parent => " children of",
                _ => "",
            };

            s += &format!(" {}", get_names(adj_in));
        }

        s += ".";
        s
    }

    fn get_descriptions(&self) -> String {
        let mut s: String = "".to_string();
        let n = self.entites.len();

        for i in 0..n-1 {
            s += &format!("{}\n", self.entity_description(i));
        }
        s += &self.entity_description(n-1);

        s
    }


    fn to_json(&self) -> String {
        let j = serde_json::to_string_pretty(self).unwrap();

        j
    }
}

impl Dot for Relations {
	fn to_dot(&self) -> String {
		let get_color = |rt: &RelationType| -> &str {
			match rt {
				&RelationType::Base => "#909090",
				&RelationType::Invoker => "red",
				&RelationType::Creator => "green",
				&RelationType::Parent => "#000000",
			}
		};

		let relation_to_dot = |i: usize, j: usize, rt: &RelationType| -> String {
			format!(
				"{} -> {} [color=\"{}\"]",
				self.entites[i].name,
				self.entites[j].name,
				get_color(rt),
			)
		};

		let mut s: String = "digraph G {\n".to_string();

		s += "\tnode [shape=record style=rounded]\n\n";

		for e in self.entites.iter() {
			s += &format!("\t{}\n", e.to_dot());
		}

		s += "\n";

		let n = self.entites.len();
		for i in 0..n {
			for j in 0..n {
				match &self.relations[i][j] {
					&Some(ref rt) => s += &format!("\t{}\n", relation_to_dot(i, j, rt)),
					&None => (),
				}
			}
		}

		s += "}";

        s
	}
}

const USAGE: &'static str = "
random-mythos

Usage:
    random-mythos (-h | --help)
    random-mythos --version
    random-mythos [--verbose=<n>]

Options:
    -h --help      Show this screen.
    --version      Show version
    --verbose=<n>  Set log level
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_verbose: usize,
    flag_version: bool,
}


fn set_logger(level: usize) {
	use simplelog::*;

    let log_level: LevelFilter = match level {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    };
	
	TermLogger::init(log_level, Config::default()).unwrap();
}


fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("random-mythos-v0.1.0");
        return;
    }

    set_logger(args.flag_verbose);

	info!("Random Mythos engage");
    io::stdout().flush().unwrap();

    print!("Number of entites to generate: ");
    io::stdout().flush().unwrap();

    let mut num = String::new();
    io::stdin().read_line(&mut num)
        .expect("Failed to read line");

    let num: usize = match num.trim().parse() {
        Ok(num) => num,
        Err(_) => panic!("Enter a number"),
    };

    let mut relations = Relations::init(num);

    relations.generate_base_relation();
    relations.generate_relations();

    write_file(&relations.to_json(), "relations.json");
    write_file(&relations.to_dot(), "relations.dot");

    let desc = relations.get_descriptions();
    write_file(&desc, "relations.md");

    println!("{}", relations.get_descriptions());
}
