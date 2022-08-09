

#![allow(non_snake_case)]
#![allow(dead_code)]


use std::path::Path;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ops;
use std::str::FromStr;
use std::time::SystemTime;

use format_num::format_num;
//use num::Complex;
use num::complex::Complex64;


fn path_to_buffer_writer(path: &Path) -> BufWriter<File> {
    let file: File = File::create(path).unwrap();
    let w: BufWriter<File> = BufWriter::new(file);
    return w;
}

fn make_png_encoder(path: &Path, size: (u32, u32)) -> png::Encoder<BufWriter<File>> {
    let encoder = png::Encoder::new(path_to_buffer_writer(path), size.0, size.1);
    return encoder;
}


fn screen_int_to_complex(int_coords: &(i32, i32), screen_size: &(i32, i32), view_size: &(f64, f64), view_corner_pos: &(f64, f64)) -> Complex64 {
    return Complex64::new((int_coords.0 as f64)*view_size.0/((screen_size.0-1) as f64)+view_corner_pos.0, (int_coords.1 as f64)*view_size.1/((screen_size.1-1) as f64)+view_corner_pos.1);
}

fn screen_complex_to_int(z: &num::complex::Complex64, screen_size: &(i32, i32), view_size: &(f64, f64), view_corner_pos: &(f64, f64)) -> (i32, i32) {
    return (((z.re-view_corner_pos.0) * (screen_size.0 as f64) / view_size.0) as i32,  ((z.im-view_corner_pos.1) * (screen_size.1 as f64) / view_size.1) as i32);
}

fn coords_to_wrapped_vec_index(int_coords: (i32, i32), wrap_width: i32) -> u64 {
    return (int_coords.1 as u64) * (wrap_width as u64) + (int_coords.0 as u64);
}



fn step_mandelbrot_point(z: Complex64, c: Complex64) -> Complex64 {
    //let zSquared = (z.re*z.re-(z.im*z.im), 2f64*z.re*z.im);
    // return (zSquared.re+c.re, zSquared.im+c.im);
    return z * z + c;
}
fn abs_squared<T: num::One + Clone + ops::Add<Output = T> + ops::Mul<Output = T>>(z: num::Complex<T>) -> T {
    // return z.re*z.re+z.im*z.im;
    num::pow(z.re, 2) + num::pow(z.im, 2)
}

struct SampleMandelbrotResult {
    iter_count: i32,
    escaped: bool,
    last_position: Complex64
}

fn sample_mandelbrot(c: Complex64, iter_limit: i32, escape_radius: f64) -> SampleMandelbrotResult {
    let mut z = Complex64::new(0.0_f64, 0.0_f64);
    let escapeRadiusSquared = num::pow(escape_radius, 2);
    for i in 0..iter_limit {
        z = step_mandelbrot_point(z, c);
        if abs_squared(z) > escapeRadiusSquared {
            return SampleMandelbrotResult {
                iter_count: i,
                escaped: true,
                last_position: z,
            };
        }
    }
    return SampleMandelbrotResult { iter_count: iter_limit, escaped: false, last_position: z };
}

fn int_is_bounded<T: PartialOrd>(value: T, min: T, max: T) -> bool {
    assert!(min < max);
    return min <= value && value <= max;
}

/*
fn lerp<T: ops::Mul<Output = T> + ops::Add<Output = T> + ops::Sub<Output = T>>(waypoint_pair: &(T, T), progress: T) -> T {
    ((waypoint_pair.1 - waypoint_pair.0) * progress) + waypoint_pair.0
}
*/
//  + ops::Div<Output = T>  / (weight_pair.0 + weight_pair.1)
fn weighted_sum_of_pair<T: ops::Mul<Output = T> + ops::Add<Output = T>>(waypoint_pair: (T, T), weight_pair: (T, T)) -> T {
    (waypoint_pair.0 * weight_pair.0) + (waypoint_pair.1 * weight_pair.1)
}


fn do_buddhabrot_point(c: Complex64, iter_limit: i32, escape_radius: f64, include_escaping: bool, include_nonescaping: bool, screen_data: &mut Vec<u8>, screen_size: &(i32, i32), view_size: &(f64, f64), view_corner_pos: &(f64, f64)) {
    let sampleResult = sample_mandelbrot(c, iter_limit, escape_radius);
    let shouldBeDrawn = (sampleResult.escaped && include_escaping) || ((!sampleResult.escaped) && include_nonescaping);

    //let mandelIndexInScreenData = coords_to_wrapped_vec_index(screenIntCoord, screen_size.0);
    //screen_data[mandelIndexInScreenData as usize] += iterCount;

    if shouldBeDrawn {
        let mut z = Complex64::new(0.0_f64, 0.0_f64);
        let mut zProgressiveSum = z.clone();
        // let escapeRadiusSquared = num::pow(escape_radius, 2);
        for i in 0..iter_limit {
            z = step_mandelbrot_point(z, c);
            zProgressiveSum += z;
            if z.norm() > escape_radius {
                return;
            }
            // assert!(z.0.abs()*0.45_f64 <= view_size.0);
            // assert!(z.1.abs()*0.45_f64 <= view_size.1);
            let escapeProgress = (i as f64) / (sampleResult.iter_count as f64);
            assert!(0.0 <= escapeProgress && escapeProgress <= 1.0);
            /*
            let modifiedZ = (
                weighted_sum_of_pair((z.0, sampleResult.last_position.0), (1.0-escapeProgress, escapeProgress)), weighted_sum_of_pair((z.1, sampleResult.last_position.1), (1.0-escapeProgress, escapeProgress))
            );
            */
            // let modifiedZ = Complex64::new(z.im*c.re+z.re*sampleResult.last_position.re, z.im*c.im+z.re*sampleResult.last_position.im);
            //let modifiedZAsComplex = Complex64 { re: modifiedZ.0, im: modifiedZ.1 };
            let zMean = zProgressiveSum / ((i + 1) as f64);
            //let modifiedZ = z;
            let screenIntCoord = screen_complex_to_int(&z, screen_size, view_size, view_corner_pos);
            
            if (!int_is_bounded(screenIntCoord.0, 0, screen_size.0-1)) || (!int_is_bounded(screenIntCoord.1, 0, screen_size.1-1)) {
                // screenIntCoord = (screenIntCoord.0 % screen_size.0, screenIntCoord.1 % screen_size.1);
                // panic!("invalid int coord reached: {:?} (from {:?} at iteration {}).", screenIntCoord, z, i);
                continue;
            }
            assert!(SCREEN_CHANNEL_COUNT == 3);
            let indexInScreenData = coords_to_wrapped_vec_index((screenIntCoord.0*(SCREEN_CHANNEL_COUNT as i32), screenIntCoord.1), screen_size.0*(SCREEN_CHANNEL_COUNT as i32)) as usize;
            let (czLen, zZmLen, cZmLen) = ((c-z).norm(), (z-zMean).norm(), (c-zMean).norm());
            let longest = (czLen>zZmLen&&czLen>cZmLen, zZmLen>czLen&&zZmLen>cZmLen, cZmLen>czLen&&cZmLen>zZmLen);
            if screen_data[indexInScreenData] <= 255-COUNT_SCALE && longest.0{
                screen_data[indexInScreenData] += COUNT_SCALE;
            }
            if screen_data[indexInScreenData+1] <= 255-COUNT_SCALE && longest.1 {
                screen_data[indexInScreenData+1] += COUNT_SCALE;
            }
            if screen_data[indexInScreenData+2] <= 255-COUNT_SCALE && longest.2 {
                screen_data[indexInScreenData+2] += COUNT_SCALE;
            }
        }
    }
}

fn itercount_to_intensity_index(itercount: i32, iterlimit: i32, intensity_limit: i32) -> i32 {
    /* intensity_limit is exclusive */
    return if itercount < iterlimit { itercount * intensity_limit / iterlimit } else { 0 };
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_int_to_complex_test() {
        let result = screen_int_to_complex(&(0,0), &(512,512), &(4.0,4.0), &(-2.0, -2.0));
        if (result.re + 2.0).abs() > 0.05 || (result.im + 2.0).abs() > 0.5 {
            panic!("low output float coord is not correct: {:?}", result);
        }

        let result = screen_int_to_complex(&(511,511), &(512,512), &(4.0,4.0), &(-2.0, -2.0));
        if (result.re - 2.0).abs() > 0.05 || (result.im - 2.0).abs() > 0.5 {
            panic!("high output float coord is not correct: {:?}", result);
        }
    }

    #[test]
    fn screen_complex_to_int_test() {
        let result = screen_complex_to_int(&Complex64::new(-1.99, -1.99), &(400,400), &(4.0,4.0), &(-2.0,-2.0));
        if result.0 > 2 || result.1 > 2 {
            panic!("low output int coord is not correct: {:?}", result);
        }
        let result = screen_complex_to_int(&Complex64::new(1.99, 1.99), &(400,400), &(4.0,4.0), &(-2.0,-2.0));
        if result.0 < 397 || result.1 < 397 {
            panic!("high output int coord is not correct: {:?}", result);
        }
    }

}






const ITER_LIMIT: i32 = 16384;
const ESCAPE_RADIUS: f64 = 2.0_f64;
// const _PALETTE_STR: &str = " .-+%#@";
// const PALETTE_SIZE: i32 = _PALETTE_STR.len() as i32;

const BIDIRECTIONAL_SUPERSAMPLING: i32 = 2;
const COUNT_SCALE: u8 = 1;
const SCREEN_SIZE: (i32, i32) = (16384, 16384);
const SEED_GRID_SIZE: (i32, i32) = (SCREEN_SIZE.0*BIDIRECTIONAL_SUPERSAMPLING, SCREEN_SIZE.1*BIDIRECTIONAL_SUPERSAMPLING);
const SCREEN_CHANNEL_COUNT: usize = 3;
// const SCREEN_PIXEL_COUNT: usize = (SCREEN_SIZE.0*SCREEN_SIZE.1) as usize;
const SCREEN_INT_COUNT: usize = ((SCREEN_SIZE.0*SCREEN_SIZE.1) as usize) * SCREEN_CHANNEL_COUNT;
const OUTPUT_BAND_COUNT: i32 = 16;
const OUTPUT_BAND_SEED_GRID_HEIGHT: i32 = SEED_GRID_SIZE.1 / OUTPUT_BAND_COUNT;



fn save_project_png(screen_data: &mut Vec<u8>, suffix: String) {
    
    // escProgressLerpsZToEscPt
    // ziAKc+zrAKescPt
    // _meanOfZSeq
    let now = SystemTime::now();

    let outfile_path_string: String = format!(
        "./output/seqs_release/test34_bb_longest(RczLenGzZmLenBcZmLen)_{itr}itr{bisuper}bisuper_color({colorScale}scale)_({width}x{height})_{suffixInsert}.png",
        itr=ITER_LIMIT, bisuper=BIDIRECTIONAL_SUPERSAMPLING, colorScale=COUNT_SCALE, width=SCREEN_SIZE.0, height=SCREEN_SIZE.1, suffixInsert=suffix,
    );
    println!("\n\nsaving an image as {}...", outfile_path_string);
    let mut encoder = make_png_encoder(Path::new(&outfile_path_string), (SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32));

    {
        let colorType: png::ColorType; // = png::ColorType::Grayscale;
        match SCREEN_CHANNEL_COUNT {
            1 => colorType = png::ColorType::Grayscale,
            3 => colorType = png::ColorType::Rgb,
            _ => panic!("unsupported channel count: {}.", SCREEN_CHANNEL_COUNT)
        };
        encoder.set_color(colorType);
    }
    encoder.set_depth(png::BitDepth::Eight);


    let mut main_writer = encoder.write_header().unwrap();
    main_writer.write_image_data(&screen_data).unwrap();
    // let elapsed = now.elapsed();
    match now.elapsed() {
        Ok(elapsed) => {
            println!("saving image took {} seconds.", format_num!(".2f", elapsed.as_secs_f32()));
        }
        Err(_e) => {
            eprintln!("calculating time taken to save image FAILED.");
        }
    }

}


fn save_project_png_seq_member(screen_data: &mut Vec<u8>, sequence_index: i32) {
    let seqIndexString = format_num!("0>4d", sequence_index);
    save_project_png(screen_data, seqIndexString);
}



fn main() {
    println!("started.");


 
    //let PALETTE: String = String::from(_PALETTE_STR);
    let view_pos = (0.0_f64, 0.0_f64);
    let view_size = (4.0_f64, 4.0_f64);
    let view_corner_pos = (view_pos.0 - 0.5*view_size.0, view_pos.1 - 0.5*view_size.1);

    let mut screen_data = vec![0_u8; SCREEN_INT_COUNT]; //: Vec<[u8; SCREEN_PIXEL_COUNT]>


    
    for y in 0..SEED_GRID_SIZE.1 {
        if y % 512 == 0 {
            print!("\n{}/{} seed rows complete.", y, SEED_GRID_SIZE.0);
        } else if y % 4 == 0 {
            print!(" {}/{}.", y, SEED_GRID_SIZE.0);
        }
        std::io::stdout().flush().unwrap();
        if y % OUTPUT_BAND_SEED_GRID_HEIGHT == 0 {
            save_project_png_seq_member(&mut screen_data, y / OUTPUT_BAND_SEED_GRID_HEIGHT);
        }
        for x in 0..SEED_GRID_SIZE.0 {
            // let intensity = (y + 2*x) % 6;
            // let currChar: String = String::from(PALETTE.as_bytes()[]);
            //let centerC = screen_int_to_float(&x, &y, &SCREEN_SIZE.0, &SCREEN_SIZE.1, &view_size.0, &view_size.1);
            //let c = (centerC.0 - view_corner_pos.0, centerC.1 - view_corner_pos.1);
            let c = screen_int_to_complex(&(x,y), &SEED_GRID_SIZE, &view_size, &view_corner_pos);
            /*
            let iterCount = sample_mandelbrot(c, ITER_LIMIT, ESCAPE_RADIUS);
            let indexInScreenData = coords_to_wrapped_vec_index((x,y), SCREEN_SIZE.0);
            screen_data[indexInScreenData as usize] = itercount_to_intensity_index(iterCount, ITER_LIMIT, 256) as u8;
            */
            do_buddhabrot_point(c, ITER_LIMIT, ESCAPE_RADIUS, true, false, &mut screen_data, &SCREEN_SIZE, &view_size, &view_corner_pos);

            // screen_data = write_to_wrapped_vec(screen_data, SCREEN_SIZE.0, (x,y), itercount_to_intensity_index(iterCount, ITER_LIMIT, 256) as u8);

        }
    }

    save_project_png(&mut screen_data, String::from_str("fin").unwrap());


}

