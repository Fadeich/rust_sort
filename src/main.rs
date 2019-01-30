
//use std::env;
use std::fs;
use std::io::prelude::*;

use structopt::StructOpt;
//#[macro_use]

//#[derive(Debug)]
#[derive(StructOpt)]
struct Cli {
	#[structopt(short = "h", long = "header")]
    header: bool,
    #[structopt(short = "col", long = "column", default_value = "2")]
    column: usize,
    #[structopt(short = "dir", long = "directory", default_value = "data")]
    dir: String,
    #[structopt(short = "o", long = "output", default_value = "foo.txt")]
    output: String,
}

fn take_position(s: &str, a: usize) -> Result<f32, &'static str> {
    let v: Vec<&str> = s.split(" ").collect();
    let result = v.get(a).ok_or("No such column!")?;
    result.parse::<f32>().map_err(|_| "Parse error")
}

fn read_data(path: String, config: &Cli) -> Result<File, &'static str> {
    let column = config.column;
    println!("Path {}", path);
    let contents = fs::read_to_string(path)
        .expect("Something went wrong reading the file");
    let mut split = contents.split("\n");
    if config.header {
        let header = split.next();
        println!("Header {}\n", header.unwrap());
    }

    let v: Result<Vec<(f32, String)>, &'static str> = split.filter(|s| !s.is_empty())
        .map(|s| match take_position(s, column) {
            Ok(a) => Ok((a, s.to_string())),
            Err(e) => Err(e)
        })
        .collect();

    match v {
        Ok(vec) => Ok(File { vec }),
        Err(e) => Err(e)
    }
}

fn write_data(line: &str, config : &Cli) -> Result<(), Box<std::error::Error>>{
	let out = &config.output;
	let mut backing_file = fs::OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .write(true)
        .open(out)?;

    backing_file.write_all(line.as_bytes())?;
    backing_file.write_all(b"\n")?;
    Ok(())
}

fn merge_files(data: Vec<File>, config : &Cli) -> Result<(), Box<std::error::Error>>{

	let num_files = data.len();
	let mut inds = vec![0; num_files];
	let mut heap = BinaryHeap::new();
	for i in 0..num_files{
		heap.push((Float(data[i].vec[0].0), i));
	}
	while heap.len() != 0 {
		let (_v, i) = heap.pop().unwrap();
		//write data to file
		let line = &data[i].vec[inds[i]].1;
        let _res = write_data(&line, &config)?;
		//println!("{:?}", v.0);
		let ind = inds[i] + 1;
		if ind < data[i].vec.len(){
			heap.push((Float(data[i].vec[ind].0), i));
			inds[i] += 1;
		}
	}
	Ok(())
}

fn sort_file(f : &mut File){ 
	f.vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
}

fn main() {
    //let args: Vec<String> = env::args().collect();
    //parse_config(&args);
    let config = Cli::from_args();
    println!("Is header: {}", config.header);
    println!("Number of column: {}", config.column);
    println!("Directory: {}", config.dir);
    println!("Output to: {}", config.output);
    
    let dir = &config.dir;
    let paths = fs::read_dir(dir).unwrap();
    let mut data: Vec<File> = Vec::new();
    for path in paths {
        let path_str = path.unwrap().path().display().to_string();
        let f = read_data(path_str, &config);
        match f {
            Ok(mut k) => {
                sort_file(&mut k);
                data.push(k);
            },
            Err(_) => {}
        };
    }
    match merge_files(data, &config){
	    Ok(()) => {},
	    Err(err) => {println!("Couldn't write to file: {}", err)},
    };
}

struct File {
    vec: Vec<(f32, String)>
}

use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(PartialEq)]
struct Float(f32);

impl Eq for Float {}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Float) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

//tests
//-------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_writes(){
    	let mut config = Cli::from_args();
    	let path = "out.txt";
    	fs::remove_file(path);
    	config.output = path.to_string();
    	let line = "there you go \n".to_string();
        let _res = write_data(&line, &config);
        let contents = fs::read_to_string(path).unwrap();
        let right_answer = "there you go \n\n";
        assert_eq!(right_answer, contents);
        fs::remove_file(path);
    }
    #[test]
    fn it_reads(){

    	let mut config = Cli::from_args();
    	let path = "out_test2.txt";
    	config.output = path.to_string();
    	config.column = 0;
    	fs::remove_file(path);
    	let line = "0.1".to_string();
        let _res = write_data(&line, &config);
        let f = read_data(path.to_string(), &config);
        let contents = f.ok().unwrap().vec[0].1.clone();
        assert_eq!(line, contents);
        fs::remove_file(path);
    }

}
