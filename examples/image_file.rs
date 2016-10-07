#[macro_use]
extern crate image;
extern crate mcq;

use std::fs::*;
use std::path::*;
use std::io::BufReader;

use mcq::MMCQ;

const COLOR_HEIGHT: u32 = 64;
const QUANT_SIZE: u32 =  16;

fn main() {
	let paths = read_dir("./examples/res");	

    for path in paths.unwrap() {
        let path = path.unwrap().path();
        let path = path.to_str().unwrap();
        process_image(path);
    }
        
    println!("\nPlease visit 'target' folder for the results");
}

fn process_image(file: &str) {
    println!("Reading image {}", file);

    let mcq = {
		let img = image::load(BufReader::new(File::open(file).unwrap()), image::JPEG).unwrap().to_rgba();    
	    let data = img.into_vec();
	
	    MMCQ::from_pixels_u8_rgba(data.as_slice(), QUANT_SIZE)    
    };
    
    let qc = mcq.get_quantized_colors();
    //println!("Quantized {:?}", qc);
    
    let img = image::load(BufReader::new(File::open(file).unwrap()), image::JPEG).unwrap().to_rgba();
    let (ix, iy) = img.dimensions();
    
    let mut imgbuf = image::ImageBuffer::new(ix, iy + COLOR_HEIGHT);
    
    for x in 0..ix {
    	for y in 0..iy {
    		imgbuf.put_pixel(x, y, img.get_pixel(x, y).clone());
    	}
    }
    
    let color_width = ix / QUANT_SIZE;
    
    for y in (iy+1)..(iy + COLOR_HEIGHT) {
		for x0 in 0..QUANT_SIZE {
			let x1 = x0 * color_width;
			let q = qc[x0 as usize];
			
	    	for x2 in 0..color_width {
	    		imgbuf.put_pixel(x1 + x2, y, image::Rgba([q.red, q.grn, q.blu, 0xff]));
	    	}
	    }	    
	}

	let ref mut fout = File::create(format!("./target/{}.png", Path::new(file).file_name().unwrap().to_str().unwrap()).as_str()).unwrap();

    let _ = image::ImageRgba8(imgbuf).save(fout, image::PNG);
}
