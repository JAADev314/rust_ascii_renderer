// use std::env;
use std::{fs};
use std::f64::consts::PI;
use clap::Parser;
use image::{ DynamicImage, GenericImage, GenericImageView, ImageResult, Pixel};
use num_traits::cast::ToPrimitive;
use num_traits::Pow;
use rayon::prelude::*;
use rand::RngExt;



//    HELPER FUNCTIONS, NOT USED 
use std::any::type_name;
fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}

struct Args {
    horizontal_samples:i32,
    vertical_samples:i32,
    horizontal_sample_size:f64,
    vertical_sample_size:f64,
    tile_width  :f64, 
    tile_height :f64, 
    offset_x    :f64, 
    offset_y    :f64, 
    ray:f64,
    collumn_offset:f64,
    sample_points:Vec<(f64,f64)>,
    negative:bool,
    contrast:f64
}

impl Args {
    fn new(width:f64, height:f64, dim_x:f64, dim_y:f64, negative:bool, contrast:f64) -> Self{
	let tile_width = width / dim_x; 
	let tile_height = height / dim_y; 
	let horizontal_samples = 3;
	let vertical_samples = 3;
	let horizontal_sample_size = tile_width/horizontal_samples as f64;
	let vertical_sample_size = tile_height/vertical_samples as f64;
	let offset_x = horizontal_sample_size / 2.0; 
	let offset_y = vertical_sample_size / 2.0;
	let collumn_offset = offset_y/2.0;
	let ray = horizontal_sample_size.max(vertical_sample_size) * 1.3;
	Self {
	    tile_width,
	    tile_height,
	    horizontal_samples,
	    vertical_samples,
	    horizontal_sample_size,
	    vertical_sample_size,
	    offset_x,
	    offset_y,
	    collumn_offset,
	    ray,
	    sample_points :         Vec::new(),
	    negative,
	    contrast,
	}
    }
}

fn get_closest_character(shape:&Vec<f64>, shapes: &Vec<Vec<f64>>, chars:&Vec<char>) -> char {
    let mut min_distance = 10000.0;
    let mut index:i32 = -1;
    // println!("-----------------------");
    // println!("shape {:#?}",shape);
    for i in 0..shapes.len(){
	let c = &shapes[i];	
	let mut distance:f64 = 0.0;
	for j in 0..c.len(){
	    distance += (shape[j]-c[j]).pow(2);
	}
	distance = distance.sqrt() as f64;
	if distance < min_distance{
	    min_distance = distance;
	    index = i as i32;
	}
    }
    // println!("{}",index);
    return chars[index as usize];
}

// generates a uniform distribution of points inside a circle
fn get_subsamples_positions(quantity:i64, ray:f64) -> Vec<(f64,f64)>{
    let mut points:Vec<(f64,f64)> = Vec::new();
    let mut rng = rand::rng();
    for _i in 0..quantity{
	// polar coordinates
	let theta:f64 = rng.random_range(0.0..1.0) * 2f64*PI;
	let mut r:f64     = rng.random_range(0.0..1.0);
	r = r.sqrt() * ray;

	//cartesian
	let x = ray * theta.sin();
	let y = ray * theta.cos();
	points.push((x,y));
    }
    return points;
}

fn read_shapes()-> Vec<Vec<f64>> {
    let contents = fs::read_to_string("shapes.txt").expect("Failed to open shapes.txt");
    let shapes: Vec<Vec<f64>> = serde_json::from_str(&contents).unwrap();
    return shapes;
}

fn lightness<T:Pixel>(pixel:T, negative:bool) -> f64{
    let r = pixel.channels()[0].to_f64().unwrap();
    let g = pixel.channels()[1].to_f64().unwrap();
    let b = pixel.channels()[2].to_f64().unwrap();
    let mut result = (r*0.2126 + g*0.7152 + b*0.0722)/255f64;
    if negative { result = 1f64 - result};
    return result
}

fn get_shape_vector<T:GenericImage>(image:&T,width:u32, height:u32, xi:f64, yi:f64, args:&Args) ->Vec<f64>{
    let mut shape:Vec<f64> = Vec::new();

    let mut max_value:f64 = 0.0;
    for i in 0..args.vertical_samples{
	for j in 0..args.horizontal_samples{
	    let mut value:f64 = 0.0;
	    let off_y = args.collumn_offset * (j%2) as f64;
	    let xi2 = j as f64 * args.horizontal_sample_size + args.offset_x + xi;
	    let yi2 = i as f64 * args.vertical_sample_size + args.offset_y + yi +off_y;
	    for k in 0..args.sample_points.len(){
		let x = (xi2 + args.sample_points[k].0).floor();
		let y = (yi2 + args.sample_points[k].1).floor();
		if x>=width as f64 || y>=height as f64 || x<0.0 || y<0.0{
		    continue;
		}
		value += lightness(image.get_pixel(x as u32,y as u32), args.negative);
	    }
	    value /= args.sample_points.len() as f64;
	    if value>max_value { max_value = value;}
	    shape.push(value);
	}
    }

    for i in 0..shape.len(){
	let mut v = shape[i];
	if v == 0.0 {continue}
	v = v/max_value;
	v = v.powf(args.contrast);
	v = v*max_value;
	shape[i] = v;
    }
    return shape;
    
}

fn generate_ascii<T>(
    image: T,
    dim_x: u32,
    dim_y: u32,
    chars: &Vec<char>,
    negative:bool,
    contrast:f64,
    letter_shapes: &Vec<Vec<f64>>,
) -> Vec<char>
where
    T: GenericImage + Sync,
{
    let width       = image.dimensions().0;
    let height      = image.dimensions().1;
    let mut args = Args::new(width as f64, height as f64, dim_x as f64, dim_y as f64, negative, contrast);
    args.sample_points = get_subsamples_positions(100, args.ray);
    (0..dim_x * dim_y)
        .into_par_iter()
        .map(|index| {
            let i = index / dim_x;
            let j = index % dim_x;

            let x = j as f64 * args.tile_width + args.offset_x;
            let y = i as f64 * args.tile_height + args.offset_y;

            let result = get_shape_vector(
                &image,
                width,
                height,
                x,
                y,
                &args,
            );

            get_closest_character(
                &result,
                letter_shapes,
                chars,
            )
        })
        .collect()
}

fn print_ascii(dim_x:u32, dim_y:u32, ascii:&Vec<char>){
    for i in 0..dim_y{
	for j in 0..dim_x{
	    print!("{}",ascii[(i*dim_x + j) as usize]);
	}
	print!("\n");
    }
}

/// Converts an image to ascii
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs{

    file_path: Option<String>,

    /// Dimensions of the ascii art (WIDTHxHEIGHT)
    #[arg(short,long)]
    dimensions: Option<String>,

    /// Inverts the clarity detection
    #[arg(short,long)]
    inverts: bool,
    
    /// Changes the contrast of ascii art. It can be bigger than 1.0
    #[arg(short,long)]
    contrast: Option<f64>,
    
}


fn parse_dimensions(dimensions:&String)->(u32, u32){
    let mut width_string  = String::new();
    let mut height_string = String::new();
    let mut switch = false;
    for c in dimensions.chars(){
	if c == 'x'{
	    switch = true;
	    continue;
	}

	if switch{ height_string.push(c);}
	else { width_string.push(c);}
    }

    let width:u32  = width_string.parse().unwrap();
    let height:u32 = height_string.parse().unwrap();
    return (width, height);
}

fn main() {
    let args = CliArgs::parse();

    
    let mut file_path = args.file_path.unwrap_or(String::from("saiki.png"));
    let mut image = image::open(file_path).unwrap();
    
    let (mut ascii_width, mut ascii_height) = match &args.dimensions {
	Some(path) => parse_dimensions(path),
        None => (image.dimensions().0/10, image.dimensions().1/10)
    };

    let shapes = read_shapes();
    let mut chars:Vec<char> = Vec::new();
    let c = args.contrast.unwrap_or(1.0);
    
    for i in 0..95{
	chars.push((i+32) as u8 as char);
    }
    let vectors = generate_ascii(image, ascii_width, ascii_height, &chars, args.inverts, c, &shapes);
    print_ascii(ascii_width,ascii_height,&vectors);
    // generate_ascii(image, 10,10);
}
// O RESULTADO ESTÁ SENDO APENAS O G. DA IMAGEM
