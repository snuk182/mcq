// A Rust port of Java implementation of Median Cut Quantization algorithm.

// This sample code is made available as part of the book "Digital Image
// Processing - An Algorithmic Introduction using Java" by Wilhelm Burger
// and Mark J. Burge, Copyright (C) 2005-2008 Springer-Verlag Berlin,
// Heidelberg, New York.
// Note that this code comes with absolutely no warranty of any kind.
// See http://www.imagingbook.com for details and licensing conditions.
//
// Date: 2007/11/10
//

// This is an implementation of Heckbert's median-cut color quantization algorithm
// (Heckbert P., "Color Image Quantization for Frame Buffer Display", ACM Transactions
// on Computer Graphics (SIGGRAPH), pp. 297-307, 1982).
// Unlike in the original algorithm, no initial uniform (scalar) quantization is used to
// for reducing the number of image colors. Instead, all colors contained in the original
// image are considered in the quantization process. After the set of representative
// colors has been found, each image color is mapped to the closest representative
// in RGB color space using the Euclidean distance.
// The quantization process has two steps: first a ColorQuantizer object is created from
// a given image using one of the constructor methods provided. Then this ColorQuantizer
// can be used to quantize the original image or any other image using the same set of
// representative colors (color table).
//

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColorDimension {
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ColorNode {
    pub rgb: u32,
    pub red: u8,
    pub grn: u8,
    pub blu: u8,
    pub cnt: usize,
}

impl ColorNode {
    fn new_rgb(rgb: u32, cnt: usize) -> ColorNode {
        ColorNode {
            rgb: (rgb & 0xFFFFFF),
            blu: ((rgb & 0xFF0000) >> 16) as u8,
            grn: ((rgb & 0xFF00) >> 8) as u8,
            red: (rgb & 0xFF) as u8,
            cnt: cnt,
        }
    }

    fn new_colors(red: u8, grn: u8, blu: u8, cnt: usize) -> ColorNode {
        ColorNode {
            rgb: ((red as u32 & 0xff) << 16) | ((grn as u32 & 0xff) << 8) | blu as u32 & 0xff,
            red: red,
            grn: grn,
            blu: blu,
            cnt: cnt,
        }
    }

    fn distance2(&self, red: u8, grn: u8, blu: u8) -> i32 {
        // returns the squared distance between (red, grn, blu)
        // and this this color
        let dr = self.red as i32 - red as i32;
        let dg = self.grn as i32 - grn as i32;
        let db = self.blu as i32 - blu as i32;
        return dr * dr + dg * dg + db * db;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct ColorBox {
    lower: usize, // lower index into 'imageColors'
    upper: usize, // upper index into 'imageColors'
    level: isize, // split level o this color box
    count: usize, // number of pixels represented by thos color box
    rmin: i32,
    rmax: i32, // range of contained colors in red dimension
    gmin: i32,
    gmax: i32, // range of contained colors in green dimension
    bmin: i32,
    bmax: i32, // range of contained colors in blue dimension
}

impl ColorBox {
    fn new(lower: usize, upper: usize, level: isize, colors: &Vec<ColorNode>) -> ColorBox {
        let mut b = ColorBox {
            lower: lower,
            upper: upper,
            level: level,

            ..Default::default()
        };

        b.trim(colors);

        b
    }

    fn color_count(&self) -> usize {
        self.upper - self.lower
    }

    fn trim(&mut self, colors: &Vec<ColorNode>) {
        // recompute the boundaries of this color box
        self.rmin = 255;
        self.rmax = 0;
        self.gmin = 255;
        self.gmax = 0;
        self.bmin = 255;
        self.bmax = 0;
        self.count = 0;
        for i in self.lower..self.upper {
            let color = colors[i];
            self.count = self.count + color.cnt;
            let r = color.red as i32;
            let g = color.grn as i32;
            let b = color.blu as i32;
            if r > self.rmax {
                self.rmax = r;
            }
            if r < self.rmin {
                self.rmin = r;
            }
            if g > self.gmax {
                self.gmax = g;
            }
            if g < self.gmin {
                self.gmin = g;
            }
            if b > self.bmax {
                self.bmax = b;
            }
            if b < self.bmin {
                self.bmin = b;
            }
        }
    }

    fn split_box(&mut self, colors: &mut Vec<ColorNode>) -> Option<ColorBox> {
        if self.color_count() < 2 {
            None // this box cannot be split
        } else {
            // find longest dimension of this box:
            let dim = self.get_longest_color_dimension();

            // find median along dim
            let med = self.find_median(dim, colors);

            // now split this box at the median return the resulting new box.
            let next_level = self.level + 1;
            let new_box = ColorBox::new(med + 1, self.upper, next_level, colors);
            self.upper = med;
            self.level = next_level;
            self.trim(colors);
            Some(new_box)
        }
    }

    fn get_longest_color_dimension(&self) -> ColorDimension {
        let r_length = self.rmax - self.rmin;
        let g_length = self.gmax - self.gmin;
        let b_length = self.bmax - self.bmin;

        if b_length >= r_length && b_length >= g_length {
            ColorDimension::Blue
        } else if g_length >= r_length && g_length >= b_length {
            return ColorDimension::Green;
        } else {
            ColorDimension::Red
        }
    }

    fn find_median(&self, dim: ColorDimension, colors: &mut Vec<ColorNode>) -> usize {
        // sort color in this box along dimension dim:
        match dim {
            ColorDimension::Red => colors[self.lower..(self.upper + 1)].sort_by(|a, b| a.red.cmp(&b.red)),
            ColorDimension::Green => colors[self.lower..(self.upper + 1)].sort_by(|a, b| a.grn.cmp(&b.grn)),
            ColorDimension::Blue => colors[self.lower..(self.upper + 1)].sort_by(|a, b| a.blu.cmp(&b.blu)),
        }

        // find the median point:
        let half = self.count / 2;
        let mut n_pixels = 0;
        // for (median = lower, n_pixels = 0; median < upper; median++) {
        for median in self.lower..self.upper {
            n_pixels = n_pixels + colors[median].cnt;
            if n_pixels >= half {
                return median;
            }
        }
        self.lower
    }

    fn get_average_color(&self, colors: &mut Vec<ColorNode>) -> ColorNode {
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        let mut n = 0usize;
        for i in self.lower..self.upper {
            let ci = colors[i];
            let cnt = ci.cnt;
            r_sum = r_sum + cnt * ci.red as usize;
            g_sum = g_sum + cnt * ci.grn as usize;
            b_sum = b_sum + cnt * ci.blu as usize;
            n = n + cnt;
        }
        // let nd = n as f64;
        let avg_red = (0.5 + r_sum as f64 / n as f64) as u8;
        let avg_grn = (0.5 + g_sum as f64 / n as f64) as u8;
        let avg_blu = (0.5 + b_sum as f64 / n as f64) as u8;
        ColorNode::new_colors(avg_red, avg_grn, avg_blu, n)
    }
}

struct ColorHistogram {
    color_array: Vec<u32>,
    count_array: Vec<usize>,
}

impl ColorHistogram {
    pub fn new(colors: Vec<u32>, counts: Vec<usize>) -> ColorHistogram {
        ColorHistogram {
            color_array: colors,
            count_array: counts,
        }
    }

    pub fn new_pixels(pixels_orig: &[u32]) -> ColorHistogram {
        let n = pixels_orig.len();
        let mut pixels_copy = Vec::with_capacity(n);
        for i in 0..n {
            // remove possible alpha components
            pixels_copy.push((0xFFFFFF & pixels_orig[i]));
        }
        pixels_copy.sort();

        // count unique colors:
        let mut k = 0; // current color index
        let mut inited = false;
        let mut cur_color = 0;
        for i in 0..pixels_copy.len() {
            if pixels_copy[i] != cur_color || !inited {
                cur_color = pixels_copy[i];
                k += 1;
                inited = true;
            }
        }

        // tabulate and count unique colors:
        let mut color_array = Vec::with_capacity(k);
        let mut count_array = Vec::with_capacity(k);
        k = 0;	// current color index
        cur_color = 0;
        let mut inited = false;
        for i in 0..pixels_copy.len() {
            if pixels_copy[i] != cur_color || !inited {
                // new color
                cur_color = pixels_copy[i];
                color_array.push(cur_color);
                count_array.push(1);
                inited = true;
                k += 1;
            } else {
                count_array[k - 1] += 1;
            }
        }
        ColorHistogram::new(color_array, count_array)
    }
}

pub struct MMCQ {
    image_colors: Vec<ColorNode>,
    quant_colors: Vec<ColorNode>,
}

impl MMCQ {
    pub fn from_pixels_u8_rgba(pixels: &[u8], k_max: u32) -> MMCQ {
        let pixels = unsafe { ::std::slice::from_raw_parts::<u32>(::std::mem::transmute(&pixels[0]), pixels.len() / 4) };

        MMCQ::from_pixels_u32_rgba(pixels, k_max)
    }

    pub fn from_pixels_u32_rgba(pixels: &[u32], k_max: u32) -> MMCQ {
        let mut m = MMCQ {
            image_colors: Vec::new(),
            quant_colors: Vec::new(),
        };

        m.quant_colors = m.find_representative_colors(&pixels, k_max);
        m.quant_colors.sort_by(|a, b| b.cnt.cmp(&a.cnt));

        m
    }

    pub fn get_quantized_colors(&self) -> &Vec<ColorNode> {
        &self.quant_colors
    }

    pub fn quantize_image(&mut self, orig_pixels: &Vec<u32>) -> Vec<u32> {
        let mut quant_pixels = orig_pixels.clone();
        for i in 0..orig_pixels.len() {
            let color = self.find_closest_color(orig_pixels[i]);
            quant_pixels[i] = color.rgb;
        }
        quant_pixels
    }

    fn find_representative_colors(&mut self, pixels: &[u32], k_max: u32) -> Vec<ColorNode> {
        let color_hist = ColorHistogram::new_pixels(pixels);
        let cnum = color_hist.color_array.len();

        self.image_colors = Vec::with_capacity(cnum);
        for i in 0..cnum {
            let rgb = color_hist.color_array[i];
            let cnt = color_hist.count_array[i];
            self.image_colors.push(ColorNode::new_rgb(rgb, cnt));
        }

        // println!("{:?}", self.image_colors);

        let r_cols = if cnum <= k_max as usize {
            // image has fewer colors than k_max
            self.image_colors.clone()
        } else {
            let initial_box = ColorBox::new(0, cnum - 1, 0, &mut self.image_colors);
            let mut color_set = Vec::new();
            color_set.push(initial_box);
            let mut k = 1;
            let mut done = false;
            while k < k_max && !done {
                let new_box = if let Some(mut next_box) = self.find_box_to_split(&mut color_set) {
                    next_box.split_box(&mut self.image_colors)
                } else {
                    done = true;
                    None
                };

                if let Some(new_box) = new_box {
                    color_set.push(new_box);
                    k = k + 1;
                }
            }

            self.average_colors(&color_set)
        };
        r_cols
    }

    fn find_closest_color(&self, rgb: u32) -> ColorNode {
        let idx = self.find_closest_color_index(rgb);
        self.quant_colors[idx]
    }

    fn find_closest_color_index(&self, rgb: u32) -> usize {
        let red = ((rgb & 0xFF0000) >> 16) as u8;
        let grn = ((rgb & 0xFF00) >> 8) as u8;
        let blu = (rgb & 0xFF) as u8;
        let mut min_idx = 0;
        let mut min_distance = ::std::i32::MAX;
        for i in 0..self.quant_colors.len() {
            let color = self.quant_colors[i];
            let d2 = color.distance2(red, grn, blu);
            if d2 < min_distance {
                min_distance = d2;
                min_idx = i;
            }
        }
        min_idx
    }

    fn average_colors(&mut self, color_boxes: &Vec<ColorBox>) -> Vec<ColorNode> {
        let n = color_boxes.len();
        let mut avg_colors = Vec::with_capacity(n);
        for b in color_boxes {
            // println!("color box {:?}", b);
            avg_colors.push(b.get_average_color(&mut self.image_colors));
            // println!("avg {:?}", avg_colors[avg_colors.len()-1]);
        }
        return avg_colors;
    }

    fn find_box_to_split<'a>(&self, color_boxes: &'a mut Vec<ColorBox>) -> Option<&'a mut ColorBox> {
        let mut box_to_split = None;
        // from the set of splitable color boxes
        // select the one with the minimum level
        let mut min_level = ::std::isize::MAX;
        for b in color_boxes {
            if b.color_count() >= 2 {
                // box can be split
                if b.level < min_level {
                    min_level = b.level;
                    box_to_split = Some(b);
                }
            }
        }
        box_to_split
    }
}
