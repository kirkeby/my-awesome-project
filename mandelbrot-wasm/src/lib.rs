mod js {
    extern {
        pub fn log(ptr: *const u8, len: usize);
        pub fn blit(ptr: *const u8, len: usize);
    }
}

fn log<S: AsRef<str>>(s: S) {
    let s = s.as_ref();
    unsafe {
        js::log(s.as_ptr(), s.len());
    }
}

const MAX_ITERATIONS: usize = 255;

#[no_mangle]
pub extern fn render(width: usize, height: usize) {
    log(format!("rendering {}x{} pixels of Mandlbrot", width, height));
    // pixel-format in u32 is AABBGGRR
    let mut buf = vec![0xff000000u32; width * height];
    let view = View::new();
    view.render(&mut buf, width, height);
    unsafe { js::blit(buf.as_ptr() as *const u8, 4 * buf.len()); }
}

#[derive(Debug)]
struct View {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl View {
    fn new() -> View {
        View {
            left: -2.5,
            right: 1.0,
            top: 1.5,
            bottom: -1.5,
        }
    }

    fn _zoom(&mut self, x: f64, y: f64) {
        let width = self.width() / 4.0;
        let height = self.height() / 4.0;
        self.left = x - width;
        self.right = x + width;
        self.top = y + height;
        self.bottom = y - height;
    }

    fn width(&self) -> f64 {
        self.right - self.left
    }

    fn height(&self) -> f64 {
        self.top - self.bottom
    }

    fn _aspect(&self) -> f64 {
        self.height() / self.width()
    }

    fn render(&self, pixels: &mut [u32], w: usize, h: usize)
    {
        let scalex = self.width() / w as f64;
        let scaley = self.height() / h as f64;

        let mut offset = 0;
        let mut cy = self.top;
        for _ in 0..h {
            let mut cx = self.left;

            for _ in 0..w {
                let c = Complex::new(cx, cy);
                let mut z = c.clone();

                let mut zz = z * z;
                for i in 0..MAX_ITERATIONS {
                    z = zz + c;
                    zz = z * z;
                    if zz.re + zz.im >= 4.0 {
                        pixels[offset] = 0xff_000000 | 0x00_010000 * (i as u32);
                        break;
                    }
                }

                cx = cx + scalex;
                offset = offset + 1;
            }

            cy = cy - scaley;
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    #[inline]
    pub fn new(re: f64, im: f64) -> Complex {
        Complex { re: re, im: im }
    }
}

impl ::std::ops::Add for Complex {
    type Output = Complex;

    #[inline]
    fn add(self, other: Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }
}

impl ::std::ops::Mul for Complex {
    type Output = Complex;

    #[inline]
    fn mul(self, other: Complex) -> Complex {
        let re = self.re.clone() * other.re.clone() - self.im.clone() * other.im.clone();
        let im = self.re * other.im + self.im * other.re;
        Complex::new(re, im)
    }
}
